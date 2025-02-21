use task_manager::TASKS_MANAGER;

pub mod task01;
pub mod task02;
pub mod task_manager;
pub mod task_stats;

#[tokio::main]
async fn main() {
    // let mut task_manager = TaskManager::default();
    TASKS_MANAGER
        .add("Task01", || Box::pin(task01::start()), 3, colored::Color::Blue)
        .await;
    TASKS_MANAGER
        .add("Task02", || Box::pin(task02::start()), 2, colored::Color::Green)
        .await;

    TASKS_MANAGER
        .add(
            "Monitor task",
            || Box::pin(task_stats::start()),
            10,
            colored::Color::Red,
        )
        .await;

    TASKS_MANAGER.list().await;
    TASKS_MANAGER.runn_tasks().await;
}
