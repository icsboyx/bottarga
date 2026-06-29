use std::io::{Write, stdout};
use std::sync::LazyLock;

use crate::common::BroadCastChannel;
use crate::tts::{TTS_QUEUE, voice_msg};
use crate::twitch_client::tw_api::{get_user_from_login, send_chat_message};
use crate::twitch_client::tw_event_sub::{self, SubscriptionTypes};
use crate::twitch_client::tw_oauth_token::{self, StreamerChannel, TW_TOKEN};
use crate::twitch_client::tw_websocket::{MessageType, WebSocketMessage};
use crate::utils::helpers::ArcRwLock;
use crate::{CONFIG_DIR, common::PersistentConfig};

use eyre::{Context, ContextCompat, Result, bail};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

static TW_WS_URL: &str = "wss://eventsub.wss.twitch.tv/ws?keepalive_timeout_seconds=30";
static TW_KEEPALIVE_MISSES_BEFORE_RECONNECT: u64 = 5;
static TW_KEEPALIVE_GRACE_SECONDS: u64 = 10;

pub static TW_BROADCASTER_CHANNEL: LazyLock<ArcRwLock<StreamerChannel>> =
    LazyLock::new(|| ArcRwLock::new(StreamerChannel::default()));
pub static TWITCH_BROADCAST: LazyLock<BroadCastChannel<TwitchChatMessage>> =
    LazyLock::new(|| BroadCastChannel::<TwitchChatMessage>::new(10));

#[derive(Debug, Clone)]
pub struct TwitchChatMessage {
    pub sender: String,
    pub payload: String,
    pub message_id: String,
}

impl TwitchChatMessage {
    pub async fn reply(&self, message: impl AsRef<str>) -> Result<()> {
        send_chat_message(message, Some(&self.message_id)).await
    }
}

// bootstrap the Twitch Broadcaster Config, broadcaster channel
pub async fn boot_tw_config() -> Result<()> {
    // Initialize the broadcaster channel
    let broadcast_channel = match broadcaster_channel_init().await {
        Ok(channel) => channel,
        Err(e) => {
            bail!("Failed to initialize broadcaster channel: {:#}", e);
        }
    };

    {
        *TW_BROADCASTER_CHANNEL.write().await = broadcast_channel;
    }
    Ok(())
}

pub async fn broadcaster_channel_init() -> Result<StreamerChannel> {
    // Load
    let mut streamer_channel = StreamerChannel::load(CONFIG_DIR).await;
    if streamer_channel != StreamerChannel::default() {
        Ok(streamer_channel)
    } else {
        log!("No streamer channel found, requesting input...");
        streamer_channel = get_streamer_channel().await?;

        streamer_channel.save(CONFIG_DIR).await;
        log!("Streamer channel saved");
        Ok(streamer_channel)
    }
}

pub async fn get_streamer_channel() -> Result<StreamerChannel> {
    print!("Enter the Twitch channel name to join: ");
    stdout().flush()?;
    let mut channel_name = String::new();
    std::io::stdin().read_line(&mut channel_name)?;
    let channel_name = channel_name.trim().to_string();
    let auth_headers = TW_TOKEN.auth_headers().await;
    let channel_info = get_user_from_login(auth_headers, &channel_name).await?;
    Ok(StreamerChannel {
        name: channel_info.login,
        id: channel_info.id,
    })
}

pub async fn start() -> Result<()> {
    log!("Twitch client started.");
    tw_oauth_token::boot_token_config().await?;
    boot_tw_config().await?;

    let (web_socket, _response) = tokio_tungstenite::connect_async(TW_WS_URL).await?;
    let (_tx, mut rx) = web_socket.split();
    let mut keepalive_timeout =
        Duration::from_secs((30 * TW_KEEPALIVE_MISSES_BEFORE_RECONNECT) + TW_KEEPALIVE_GRACE_SECONDS);

    loop {
        let msg = timeout(keepalive_timeout, rx.next())
            .await
            .map_err(|_| eyre::eyre!("No Twitch WebSocket message received for {:?}", keepalive_timeout))?
            .ok_or_else(|| eyre::eyre!("Twitch WebSocket stream ended"))??;

        match msg {
            Message::Text(data) => {
                let msg = serde_json::from_slice::<WebSocketMessage>(&data.as_bytes()).context(here!())?;

                match msg.metadata.message_type {
                    MessageType::SessionWelcome => {
                        keepalive_timeout = manage_session_welcome(msg).await?;
                    }
                    MessageType::SessionKeepAlive => manage_session_keepalive(msg).await?,
                    MessageType::Notification => manage_notification(msg).await?,
                    MessageType::SessionReconnect => manage_session_reconnect(msg).await?,
                    MessageType::Revocation => manage_revocation(msg).await?,
                }
            }
            Message::Ping(data) => {
                log!("Received ping message: ({})", String::from_utf8_lossy(&data));
            }
            Message::Close(data) => {
                return Err(eyre::eyre!("Received close message: {:#?}", data));
            }
            _ => {}
        }
    }
}

pub async fn manage_session_welcome(msg: WebSocketMessage) -> Result<Duration> {
    let streamer_id = TW_BROADCASTER_CHANNEL.get_clone().await.id;
    let session = msg.payload.session.as_ref().context(here!())?;
    let session_id = session.id.as_str();
    log!("Handling session welcome message, id {}", session_id);
    tw_event_sub::sub_channel_chat_message(&streamer_id, session_id).await?;
    tw_event_sub::sub_user_whisper_message(&streamer_id, session_id).await?;
    Ok(Duration::from_secs(
        (session.keepalive_timeout_seconds.unwrap_or(30) * TW_KEEPALIVE_MISSES_BEFORE_RECONNECT)
            + TW_KEEPALIVE_GRACE_SECONDS,
    ))
}

pub async fn manage_session_keepalive(msg: WebSocketMessage) -> Result<()> {
    log!(
        "Received keep-alive message: {} id: {}",
        msg.metadata.message_timestamp,
        msg.metadata.message_id
    );
    Ok(())
}

pub async fn manage_notification(msg: WebSocketMessage) -> Result<()> {
    // log!("Handling notification message: {:#}", serde_json::to_value(&msg)?);
    if let Some(subscription) = msg.payload.subscription {
        match subscription.subscription_type {
            SubscriptionTypes::ChannelChatMessage => {
                if let Some(event) = msg.payload.event {
                    let Some(chatter_id) = event.get("chatter_user_id") else {
                        log!("No chatter_user_id found in event, ignoring message");
                        return Ok(());
                    };

                    let Some(chatter_name) = event.get("chatter_user_login") else {
                        log!("No chatter_user_login found in event, ignoring message");
                        return Ok(());
                    };

                    let Some(message) = event.get("message").and_then(|m| m.get("text")) else {
                        log!("No message text found in event, ignoring message");
                        return Ok(());
                    };

                    if chatter_id == &TW_TOKEN.user_id().await {
                        log!("Received own chat message, ignoring");
                        return Ok(());
                    }

                    let chat_message = TwitchChatMessage {
                        sender: chatter_name.as_str().unwrap_or_default().to_string(),
                        payload: message.as_str().unwrap_or_default().to_string(),
                        message_id: event["message_id"].as_str().unwrap_or_default().to_string(),
                    };

                    TWITCH_BROADCAST.send_broadcast(chat_message.clone()).await?;
                    TTS_QUEUE
                        .push_back(voice_msg(&chat_message.payload, &chat_message.sender).await)
                        .await;

                    log!("[ {} ]: {}", chat_message.sender, chat_message.payload);
                }
            }

            SubscriptionTypes::UserWhisperMessage => {
                if let Some(event) = msg.payload.event {
                    log!("Received whisper message: {:?}", event);
                }
            }
        }
    }

    Ok(())
}

pub async fn manage_session_reconnect(msg: WebSocketMessage) -> Result<()> {
    let reconnect_url = msg
        .payload
        .session
        .as_ref()
        .and_then(|session| session.reconnect_url.as_deref())
        .unwrap_or("missing reconnect_url");
    Err(eyre::eyre!(
        "Twitch requested WebSocket reconnect to {}. Restarting Twitch client task.",
        reconnect_url
    ))
}

pub async fn manage_revocation(msg: WebSocketMessage) -> Result<()> {
    log!("Handling revocation message: {:#?}", msg);
    Ok(())
}
