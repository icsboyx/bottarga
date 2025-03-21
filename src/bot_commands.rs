use std::collections::HashMap;
use std::fmt::Display;
use std::pin::Pin;
use std::sync::LazyLock;

use anyhow::{Error, Result};
use tokio::sync::RwLock;

use crate::irc_parser::IrcMessage;
use crate::tts::{TTS_QUEUE, voice_msg};
use crate::twitch_client::{IntoIrcPRIVMSG, TWITCH_BOT_INFO, TWITCH_BROADCAST, TWITCH_RECEIVER};

pub static BOT_COMMAND_PREFIX: &str = "!";

pub static BOT_COMMANDS: LazyLock<BotCommands> = LazyLock::new(|| BotCommands::default());
type BotCommandType = fn(IrcMessage) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>>;

#[derive(Default)]
pub struct BotCommands {
    commands: RwLock<HashMap<String, BotCommandType>>,
}

// impl<T: Display> IntoIrcPRIVMSG for T {}

impl BotCommands {
    pub async fn add_command(&self, trigger: impl Into<String>, command: BotCommandType) {
        let trigger = trigger.into();
        log_debug!("Adding command: {}", trigger);
        self.commands.write().await.insert(trigger, command);
    }

    pub async fn run_command(&self, command: &str, message: IrcMessage) -> Result<()> {
        log_trace!("Running command: {}", command);
        if let Some(func) = self.commands.read().await.get(command) {
            func(message).await?;
        }
        Ok(())
    }
}

pub async fn start() -> Result<()> {
    let mut test_broadcast_rx = TWITCH_BROADCAST.subscribe_broadcast().await;

    BOT_COMMANDS
        .add_command("help", |irc_message| Box::pin(list_all_commands(irc_message)))
        .await;

    BOT_COMMANDS
        .add_command("test", |irc_message| Box::pin(test_command(irc_message)))
        .await;

    BOT_COMMANDS
        .add_command("die", |irc_message| Box::pin(die(irc_message)))
        .await;

    // Read all broadcasted commands from Twitch_client
    while let Ok(ret_val) = test_broadcast_rx.recv().await {
        match ret_val.command.as_str() {
            "PRIVMSG" if ret_val.payload.starts_with(BOT_COMMAND_PREFIX) => {
                let command = ret_val
                    .payload
                    .split_whitespace()
                    .next()
                    .unwrap()
                    .trim_start_matches(BOT_COMMAND_PREFIX);
                BOT_COMMANDS.run_command(command, ret_val.clone()).await?;
            }
            _ => {}
        };
    }

    Ok(())
}

impl<T: Display> IntoIrcPRIVMSG for T {}

pub async fn die(_message: IrcMessage) -> Result<()> {
    let ret_val = "Goodbye cruel world";
    TTS_QUEUE
        .push_back(voice_msg(&ret_val, &TWITCH_BOT_INFO.nick_name().await).await)
        .await;
    TWITCH_RECEIVER.push_back(ret_val.as_irc_privmsg().await).await;
    futures::future::err(Error::msg("I'm dying as you wish!")).await
}

pub async fn test_command(message: IrcMessage) -> Result<()> {
    let ret_val = format!("Hi there {} this is the reply to your test message", message.sender);
    TTS_QUEUE
        .push_back(voice_msg(&ret_val, &TWITCH_BOT_INFO.nick_name().await).await)
        .await;
    TWITCH_RECEIVER.push_back(ret_val.as_irc_privmsg().await).await;
    Ok(())
}

pub async fn list_all_commands(_message: IrcMessage) -> Result<()> {
    let triggers = BOT_COMMANDS
        .commands
        .read()
        .await
        .iter()
        .map(|(trigger, _)| format!("{}{}", BOT_COMMAND_PREFIX, trigger))
        .collect::<Vec<_>>()
        .join(", ");

    let ret_val = format!("Available commands: {}", triggers);
    TTS_QUEUE
        .push_back(voice_msg(&ret_val, &TWITCH_BOT_INFO.nick_name().await).await)
        .await;
    TWITCH_RECEIVER.push_back(ret_val.as_irc_privmsg().await).await;
    Ok(())
}
