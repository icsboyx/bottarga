use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use curl::easy::Easy;
use futures::executor::block_on;
use serde::{Deserialize, Serialize};

use crate::CONFIG_DIR;
use crate::audio_player::TTS_AUDIO_QUEUE;
use crate::bot_commands::{BOT_COMMAND_PREFIX, BOT_COMMANDS};
use crate::defs::PersistentConfig;
use crate::irc_parser::IrcMessage;
use crate::tts::{TTS_QUEUE, voice_msg};
use crate::twitch_client::{IntoIrcPRIVMSG, TWITCH_BOT_INFO, TWITCH_RECEIVER};

pub static EXTERNAL_COMMANDS_FILE: &str = "ExternalBotCommands.toml";

#[derive(Deserialize, Debug, Clone, Serialize)]
struct ExternalBotCommand {
    activation_pattern: String,
    need_arg: bool,
    custom_audio_url: String,
    replay_text: String,
    // play_mode: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct ExternalBotCommands {
    commands: HashMap<String, ExternalBotCommand>,
}

impl PersistentConfig for ExternalBotCommands {}

impl Default for ExternalBotCommands {
    fn default() -> Self {
        let cmd_test = ExternalBotCommand {
            activation_pattern: "test".into(),
            need_arg: false,
            custom_audio_url: "".into(),
            replay_text: "Hi there {SENDER} this is the reply to your test command".to_string(),
        };

        let cmd_meow = ExternalBotCommand {
            activation_pattern: "meow".into(),
            need_arg: false,
            custom_audio_url: "https://www.myinstants.com/media/sounds/m-e-o-w.mp3".into(),
            replay_text: "".into(),
        };

        let cmd_for_president: ExternalBotCommand = ExternalBotCommand {
            activation_pattern: "for_president".into(),
            need_arg: true,
            custom_audio_url: "".into(),
            replay_text: "{ARG} for President!".into(),
        };

        let mut commands = HashMap::new();
        commands.insert("test".into(), cmd_test);
        commands.insert("meow".into(), cmd_meow);
        commands.insert("for_president".into(), cmd_for_president);

        Self { commands }
    }
}

impl ExternalBotCommands {
    pub fn init() -> Self {
        let ret_val = block_on(async { ExternalBotCommands::load(CONFIG_DIR).await });
        ret_val
    }

    pub async fn reg_ext_bot_cmd(&self) -> Result<()> {
        for ext_cmd in self.commands.values() {
            ext_bot_cmd(ext_cmd.clone()).await?;
        }
        Ok(())
    }
}

async fn ext_bot_cmd(command: ExternalBotCommand) -> Result<()> {
    if command.activation_pattern.is_empty() {
        log_error!("Activation command is empty, skipping command {:?}", &command);
        return Ok(());
    }

    BOT_COMMANDS
        .add_command(
            command.activation_pattern.clone(),
            Arc::new(move |irc_message| {
                let command = command.clone();
                Box::pin(handle_command(irc_message, command))
            }),
        )
        .await;

    Ok(())
}
async fn get_audio_data(url: impl AsRef<str>) -> Vec<u8> {
    let mut data = Vec::new();
    let mut easy = Easy::new();
    let _ = easy.ssl_verify_peer(false);
    easy.url(url.as_ref()).unwrap();
    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }
    data
}

// let ret_val = irc_message.payload.split_once(" ");
// if command.need_payload && ret_val.is_some() && !ret_val.unwrap().1.is_empty() {
//     let payload = command.replay_text.replace("{ARG}", ret_val.unwrap().1);
//     // Simulated async operations
//     let _ = voice_msg(&payload, &"Bot".to_string()).await;
//     let _ = TWITCH_RECEIVER.push_back(vec![payload]).await;
// }

// if !command.custom_audio_url.is_empty() {
//     // Simulated async operation for custom audio URL
//     let _ = TTS_AUDIO_QUEUE
//         .push_back(get_audio_data(command.custom_audio_url).await)
//         .await;
// }

async fn handle_command(irc_message: IrcMessage, command: ExternalBotCommand) -> Result<()> {
    log_debug!("Running command: {}", command.activation_pattern);

    let reply_payload = if command.need_arg {
        if let Some(arg) = irc_message.payload.split_once(" ").map(|(_, arg)| arg) {
            command.replay_text.replace("{ARG}", arg)
        } else {
            format!(
                "Hey @{}, you need to provide an argument for {}{} command",
                &irc_message.sender, BOT_COMMAND_PREFIX, &command.activation_pattern
            )
        }
    } else {
        command.replay_text.replace("{SENDER}", &irc_message.sender)
    };

    if !command.custom_audio_url.is_empty() {
        let audio_data = get_audio_data(&command.custom_audio_url).await;
        TTS_AUDIO_QUEUE.push_back(audio_data).await;
    }

    TTS_QUEUE
        .push_back(voice_msg(&reply_payload, &TWITCH_BOT_INFO.nick_name().await).await)
        .await;
    TWITCH_RECEIVER.push_back(reply_payload.as_irc_privmsg().await).await;

    Ok(())
}
