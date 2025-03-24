use anyhow::Result;
use bot_external_commands::ExternalBotCommands;
use task_manager::TASKS_MANAGER;
pub mod defs;
#[macro_use]
pub mod macros;
pub mod audio_player;
pub mod bot_commands;
pub mod bot_external_commands;
pub mod irc_parser;
pub mod task_manager;
pub mod task_stats;
pub mod tts;
pub mod twitch_client;
pub mod users;

pub static CONFIG_DIR: Option<&'static str> = Some(".config");

#[tokio::main]

async fn main() -> Result<()> {
    // Start the Twitch client
    TASKS_MANAGER
        .add("Task01", || Box::pin(twitch_client::start()), 3)
        .await;
    // Start the TTS client
    TASKS_MANAGER.add("TTS", || Box::pin(tts::start()), 3).await;
    // Start the Audio Player
    TASKS_MANAGER
        .add("AUDIO_PLAYER", || Box::pin(audio_player::start()), 3)
        .await;
    // Start the Bot Commands
    TASKS_MANAGER
        .add("BOT_COMMANDS", || Box::pin(bot_commands::start()), 3)
        .await;

    let ret_val = ExternalBotCommands::init();
    log_trace!("Test External Bot Command {:?}", ret_val);
    // List all tasks
    TASKS_MANAGER.list().await;
    // Start all tasks
    TASKS_MANAGER.run_tasks().await;

    Ok(())
}
