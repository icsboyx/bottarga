use std::fmt::Display;
use std::sync::LazyLock;
use std::time::Duration;

use anyhow::{Result, anyhow};
use futures::executor::block_on;
use futures::{SinkExt, StreamExt, pin_mut};
use msedge_tts::tts::SpeechConfig;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message;

use crate::bot_commands::BOT_COMMAND_PREFIX;
use crate::common::{BroadCastChannel, MSGQueue, PersistentConfig};
use crate::irc_parser::{IrcMessage, parse_message};
use crate::tts::{TTS_QUEUE, TTS_VOCE_BD, voice_msg};
use crate::{CONFIG_DIR, log};

pub static TWITCH_BOT_INFO: LazyLock<TwitchBotInfo> = LazyLock::new(|| TwitchBotInfo::init());
pub static TWITCH_BROADCAST: LazyLock<BroadCastChannel<IrcMessage>> =
    LazyLock::new(|| BroadCastChannel::<IrcMessage>::new(10));
pub static TWITCH_RECEIVER: LazyLock<MSGQueue<Vec<String>>> = LazyLock::new(|| MSGQueue::new());

static TWITCH_MAX_MSG_LINE_LENGTH: usize = 400;

pub(crate) trait IntoIrcPRIVMSG {
    async fn as_irc_privmsg(&self) -> Vec<String>
    where
        Self: Display + AsRef<str>,
    {
        split_lines(self).await.fold(Vec::<String>::new(), |mut lines, line| {
            lines.push(block_on(async {
                format!("PRIVMSG {} :{}", TWITCH_BOT_INFO.channel().await, line)
            }));
            lines
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BotSpeechConfig {
    speech_config: SpeechConfig,
}
impl Default for BotSpeechConfig {
    fn default() -> Self {
        BotSpeechConfig {
            speech_config: TTS_VOCE_BD.filter_voices_by_text(&["it-IT", "multi"]).random().into(),
        }
    }
}

impl PersistentConfig for BotSpeechConfig {}

impl BotSpeechConfig {
    pub async fn init() -> Self {
        BotSpeechConfig::load(CONFIG_DIR).await
    }
}

pub struct TwitchBotInfo {
    nick_name: RwLock<String>,
    channel: RwLock<String>,
    speech_config: SpeechConfig,
}

impl TwitchBotInfo {
    pub fn init() -> Self {
        TwitchBotInfo {
            nick_name: RwLock::new("justinfan69696942".into()),
            channel: RwLock::new("icsboyx".into()),
            speech_config: block_on(async { BotSpeechConfig::init().await.speech_config }),
        }
    }

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

    pub async fn speech_config(&self) -> &SpeechConfig {
        &self.speech_config
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
                "twitch.tv/membership".into(),
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
                for line in ret_val {
                    log_debug!("SENDING: {:?}", line);
                let _ = write.send(line.to_ws_text()).await;
            }

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

    TWITCH_RECEIVER.push_back(auth).await;

    Ok(())
}

pub async fn split_lines(message: impl AsRef<str>) -> impl Iterator<Item = String> {
    let messages = message
        .as_ref()
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
        match line.command.as_str() {
            "PING" => {
                log_debug!("Replying to Server Ping");
                TWITCH_RECEIVER
                    .push_back([format!("PONG :{}", line.payload)].to_vec())
                    .await;
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
                TWITCH_BROADCAST.send_broadcast(line.clone()).await?;
                if !line.payload.starts_with(BOT_COMMAND_PREFIX) {
                    TTS_QUEUE.push_back(voice_msg(&line.payload, &line.sender).await).await;
                }
            }
            "PONG" => {
                log_debug!("Received PONG from server");
            }
            _ => {}
        }
    }

    Ok(())
}
