use anyhow::Result;
use task_manager::TASKS_MANAGER;
pub mod defs;
#[macro_use]
pub mod macros;
pub mod irc_parser;
pub mod task_manager;
pub mod task_stats;
pub mod twitch_client;

pub static CONFIG_DIR: Option<&'static str> = Some(".config");

use colored::Color::*;
#[tokio::main]

async fn main() -> Result<()> {
    // let mut task_manager = TaskManager::default();

    log_debug!("{:?}", TASKS_MANAGER);

    TASKS_MANAGER
        .add("Task01", || Box::pin(twitch_client::start()), 3, Some(Blue))
        .await;

    log_debug!("{:?}", TASKS_MANAGER);

    TASKS_MANAGER.list().await;
    TASKS_MANAGER.run_tasks().await;
    Ok(())
}
