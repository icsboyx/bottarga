use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, LazyLock};

use eyre::{Error, Result};
use tokio::sync::RwLock;

use crate::bot_external_commands::ExternalBotCommands;
use crate::tts::{TTS_QUEUE, voice_msg};
use crate::twitch_client::tw_client::{TWITCH_BROADCAST, TwitchChatMessage};
use crate::twitch_client::tw_oauth_token::TW_TOKEN;

pub static BOT_COMMAND_PREFIX: &str = "!";

pub static BOT_COMMANDS: LazyLock<BotCommands> = LazyLock::new(|| BotCommands::default());

pub type BotCommandType =
    Arc<dyn Fn(TwitchChatMessage) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Sync + Send>> + Sync + Send>;

#[derive(Default)]
pub struct BotCommands {
    commands: RwLock<HashMap<String, BotCommandType>>,
}

impl BotCommands {
    pub async fn add_command(&self, trigger: impl Into<String>, command: BotCommandType) {
        let trigger = trigger.into();
        log_debug!("Adding command: {}", trigger);
        self.commands.write().await.insert(trigger, command);
    }

    pub async fn run_command(&self, command: &str, message: TwitchChatMessage) -> Result<()> {
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
        .add_command(
            "help",
            Arc::new(|chat_message| Box::pin(bot_cmd_list_all_commands(chat_message))),
        )
        .await;

    // BOT_COMMANDS
    //     .add_command("die", Arc::new(|chat_message| Box::pin(die(chat_message))))
    //     .await;

    let ext_bot_commands = ExternalBotCommands::init();
    ext_bot_commands.reg_ext_bot_cmd().await?;

    // Read all broadcasted commands from Twitch_client
    while let Ok(ret_val) = test_broadcast_rx.recv().await {
        if ret_val.payload.starts_with(BOT_COMMAND_PREFIX) {
            let command = ret_val
                .payload
                .split_whitespace()
                .next()
                .unwrap()
                .trim_start_matches(BOT_COMMAND_PREFIX);
            BOT_COMMANDS.run_command(command, ret_val.clone()).await?;
        }
    }

    Ok(())
}

pub async fn bot_cmd_list_all_commands(message: TwitchChatMessage) -> Result<()> {
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
        .push_back(voice_msg(&ret_val, &TW_TOKEN.login().await).await)
        .await;
    message.reply(ret_val).await?;
    Ok(())
}
