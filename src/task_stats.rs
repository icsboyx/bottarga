use std::time::Duration;

use anyhow::Result;

use crate::task_manager::TASKS_MANAGER;

pub async fn start() -> Result<()> {
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        TASKS_MANAGER.get_stats().await;
    }
}
