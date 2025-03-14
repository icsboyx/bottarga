use std::fmt::Display;
use std::sync::LazyLock;
use std::time::Duration;

use anyhow::{Result, anyhow};
use futures::{SinkExt, StreamExt, pin_mut};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message;

use crate::defs::{BroadCastChannel, MSGQueue, PersistentConfig};
use crate::irc_parser::{IrcMessage, parse_message};
use crate::{CONFIG_DIR, log};

static TWITCH_MAX_MSG_LINE_LENGTH: usize = 500;
pub static TWITCH_BOT_INFO: LazyLock<TwitchBotInfo> = LazyLock::new(|| TwitchBotInfo::default());
pub static TWITCH_BROADCAST: LazyLock<BroadCastChannel<IrcMessage>> =
    LazyLock::new(|| BroadCastChannel::<IrcMessage>::new("Twitch Broadcast channel", 10));
pub static TWITCH_RECEIVER: LazyLock<MSGQueue<String>> = LazyLock::new(|| MSGQueue::<String>::default());

pub(crate) trait IntoIrcPRIVMSG {
    async fn into_privmsg(&self) -> String
    where
        Self: Display,
    {
        format!("PRIVMSG {} :{}", TWITCH_BOT_INFO.channel().await, self)
    }
}

impl<T> IntoIrcPRIVMSG for T {}

#[derive(Default)]
pub struct TwitchBotInfo {
    nick_name: RwLock<String>,
    channel: RwLock<String>,
}

impl TwitchBotInfo {
    pub async fn nick_name(&self) -> String {
        self.nick_name.read().await.to_string()
    }

    pub async fn set_nickname(&self, nick_name: impl AsRef<str>) {
        *self.nick_name.write().await = nick_name.as_ref().into();
    }

    pub async fn channel(&self) -> String {
        self.channel.read().await.to_string()
    }

    pub async fn set_channel(&self, channel: impl AsRef<str>) {
        *self.channel.write().await = channel.as_ref().into();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TwitchConfig {
    server: String,
    nick: String,
    channel: String,
    auth_token: String,
    irc_cap_req: Vec<String>,
    ping_interval: u64,
}

impl Default for TwitchConfig {
    fn default() -> Self {
        TwitchConfig {
            server: "irc.chat.twitch.tv".into(),
            nick: "justinfan69696942".into(),
            channel: "icsboyx".into(),
            auth_token: "1234567890".into(),
            irc_cap_req: vec![
                "twitch.tv/commands".into(),
                "twitch.tv/commands".into(),
                "twitch.tv/tags".into(),
            ],
            ping_interval: 180,
        }
    }
}

impl PersistentConfig for TwitchConfig {}

trait WsMessageHandler {
    fn to_ws_text(&self) -> Message;
}

impl<T> WsMessageHandler for T
where
    T: std::fmt::Display + Into<String>,
{
    fn to_ws_text(&self) -> Message {
        Message::text(self.to_string())
    }
}

pub async fn start() -> Result<()> {
    log!("Starting Twitch client");
    let twitch_config = TwitchConfig::load(CONFIG_DIR).await;

    let (websocket, _response) = tokio_tungstenite::connect_async("wss://irc-ws.chat.twitch.tv:443").await?;
    let (mut write, mut read) = websocket.split();

    twitch_auth(&twitch_config).await?;

    let ping_interval = tokio::time::interval(Duration::from_secs(twitch_config.ping_interval));
    pin_mut!(ping_interval);
    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                log_debug!("Sending PING to twitch server");
                let payload = "PING :tmi.twitch.tv";
                write.send(payload.to_ws_text()).await?;
            }

            ret_val = read.next() => {
                match ret_val {
                    Some(Ok(msg)) => {
                        match msg {
                            Message::Text(text) => {
                                handle_twitch_msg(text.trim()).await?;
                            }
                            Message::Close(close) => {
                                log!("Connection closed: {:?}", close);
                                break;
                            }
                            _ => {
                                log!("Received non-text message: {:?}", msg);
                            }
                        }
                    }
                    Some(Err(e)) => {
                        log_error!("Error reading message: {}", e);
                    }
                    None => {
                        log_error!("Connection closed");
                        break;
                    }
                }
            }

            Some(ret_val) = TWITCH_RECEIVER.next() => {
                log_debugc!(BrightCyan, "SENDING: {:?}", ret_val);
                let _ = write.send(ret_val.to_ws_text()).await;
            }




        }
    }
    Ok(())
}

async fn twitch_auth(config: &TwitchConfig) -> Result<()> {
    let mut auth = vec![
        format!("PASS oauth:{}", config.auth_token),
        format!("NICK {}", config.nick),
    ];
    for cap in &config.irc_cap_req {
        auth.push(format!("CAP REQ :{}", cap));
    }
    auth.push(format!("JOIN #{}", config.channel));

    for msg in auth {
        TWITCH_RECEIVER.push_back(msg).await;
    }
    Ok(())
}

pub async fn split_message(message: impl Into<String>) -> impl Iterator<Item = String> {
    let messages = message
        .into()
        .split_whitespace()
        .fold(Vec::new(), |mut lines: Vec<String>, word| {
            if let Some(last) = lines.last_mut() {
                if last.len() + word.len() + 1 <= TWITCH_MAX_MSG_LINE_LENGTH {
                    last.push(' ');
                    last.push_str(word);
                } else {
                    lines.push(word.to_string());
                }
            } else {
                lines.push(word.to_string());
            }
            lines
        });

    messages.into_iter()
}

async fn handle_twitch_msg(text: impl AsRef<str>) -> Result<()> {
    let text = text.as_ref();
    let lines = text.split('\n').map(|line| parse_message(line)).collect::<Vec<_>>();
    // log!("{:?}", lines);

    for line in lines {
        // log_debugc!(Green, "RECEIVING: {:?}", line);
        match line.command.as_str() {
            "PING" => {
                log_debug!("Replying to Server Ping");
                TWITCH_RECEIVER.push_back(format!("PONG :{}", line.payload)).await;
            }
            "PRIVMSG" if line.payload == "!die" => {
                log_error!("I'm dying cruel world");
                return Result::Err(anyhow!("I'm dying cruel world"));
            }
            "001" => {
                // First reply, you can use destination as bot NickName
                log!("Bot NickName is: {}", line.destination);
                TWITCH_BOT_INFO.set_nickname(line.destination).await;
            }
            "JOIN" => {
                // Joining channel, you can use destination as channel name
                log!("Joined channel: {}", line.destination);
                TWITCH_BOT_INFO.set_channel(line.destination).await;
            }
            "PRIVMSG" => {
                TWITCH_BROADCAST.send_broadcast(line).await?;
            }
            _ => {}
        }
    }

    Ok(())
}
