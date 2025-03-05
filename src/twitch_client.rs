use std::sync::LazyLock;
use std::time::Duration;

use anyhow::{Error, Result, anyhow};
use futures::{SinkExt, StreamExt, pin_mut};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};

use crate::defs::PersistentConfig;
use crate::irc_parser::parse_message;
use crate::{CONFIG_DIR, irc_parser, log};

static TWITCH_MAX_MSG_LINE_LENGTH: usize = 500;
static TWITCH_BOT_INFO: LazyLock<TwitchBotInfo> = LazyLock::new(|| TwitchBotInfo::default());

#[derive(Default)]
struct TwitchBotInfo {
    NickName: RwLock<String>,
    Channel: RwLock<String>,
}

impl TwitchBotInfo {
    async fn nick_name(&self) -> String {
        self.NickName.read().await.to_string()
    }

    async fn set_nickname(&self, nick_name: impl AsRef<str>) {
        *self.NickName.write().await = nick_name.as_ref().into();
    }

    async fn channel(&self) -> String {
        self.Channel.read().await.to_string()
    }

    async fn set_channel(&self, channel: impl AsRef<str>) {
        *self.Channel.write().await = channel.as_ref().into();
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

    for msg in twitch_auth(&twitch_config) {
        write.send(msg).await?;
    }

    let ping_interval = tokio::time::interval(Duration::from_secs(twitch_config.ping_interval));
    pin_mut!(ping_interval);
    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
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

        }
    }
    Ok(())
}

fn twitch_auth(config: &TwitchConfig) -> impl Iterator<Item = Message> {
    let mut auth = vec![format!("PASS {}", config.auth_token), format!("NICK {}", config.nick)];
    for cap in &config.irc_cap_req {
        auth.push(format!("CAP REQ :{}", cap));
    }
    auth.push(format!("JOIN #{}", config.channel));
    auth.into_iter().map(|msg| Message::text(msg))
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
        match line.command.as_str() {
            "PING" => log_debug!("Server is pinging us"),
            "PRIVMSG" if line.payload == "!die" => {
                log_error!("Voi che muoro");
                return Result::Err(anyhow!("dddddddddd"));
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
            "PRIVMSG" => log!("{}", line.payload),
            _ => {
                // log_warning!("Command not managed: {:?}", line)
            }
        }
    }

    Ok(())
}
