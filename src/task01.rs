use std::time::Duration;

use anyhow::{Ok, Result};

pub async fn start() -> Result<()> {
    let mut execution_number = 0;
    loop {
        execution_number += 1;
        println!("{} ripetition {}", "task 1", execution_number);
        tokio::time::sleep(Duration::from_secs(1)).await;
        if execution_number == 5 {
            println!("exiting task 1");
            break;
        }
    }
    Ok(())
}
