use std::fmt::Debug;
use std::pin::Pin;
use std::sync::LazyLock;
use std::time::Duration;

use anyhow::{Error, Result};
use futures::executor::block_on;
use futures::stream::StreamExt;
use futures::{pin_mut, stream};
use tokio::sync::RwLock;

pub static TASKS_MANAGER: LazyLock<TaskManager> = LazyLock::new(|| TaskManager::default());
static TASK_MONITOR_TIME: u64 = 600;

// type BotTaskType = dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>>;

#[derive(Debug, Default, Clone)]
pub struct BotTaskStatus {
    max_restarts: i32,
    restart_status: i32,
    is_alive: bool,
}

impl BotTaskStatus {
    pub async fn set_max_restarts(&mut self, max_restarts: i32) {
        self.max_restarts = max_restarts;
    }

    pub async fn set_restart_status(&mut self, restart_status: i32) {
        self.restart_status = restart_status;
    }

    pub async fn get_stats(&self) -> BotTaskStatus {
        self.clone()
    }

    pub fn is_alive(&self) -> Result<()> {
        let _ = self.is_alive;
        Ok(())
    }
}

impl std::fmt::Display for BotTaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

type BotTaskType = fn() -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>>;
#[derive(Debug)]
pub struct BotTask {
    name: String,
    function: RwLock<BotTaskType>,
    task_status: RwLock<BotTaskStatus>,
}

impl BotTask {
    pub fn new(name: impl AsRef<str>, task: BotTaskType, max_restarts: i32) -> Self {
        Self {
            name: name.as_ref().into(),
            function: RwLock::new(task),
            task_status: RwLock::new(BotTaskStatus {
                max_restarts,
                restart_status: 0,
                is_alive: false,
            }),
        }
    }

    pub async fn run(&self) -> Result<()> {
        (self.function.read().await)().await
    }
}

#[derive(Debug)]
pub struct TaskManager {
    pub tasks: RwLock<Vec<BotTask>>,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self {
            tasks: RwLock::new(vec![BotTask {
                name: "TaskMonitor".into(),
                function: RwLock::new(|| Box::pin(task_monitor())),
                task_status: RwLock::new(BotTaskStatus {
                    max_restarts: -1,
                    restart_status: 0,
                    is_alive: false,
                }),
            }]),
        }
    }
}

impl std::fmt::Display for TaskManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self.tasks)
    }
}

impl TaskManager {
    // pub async fn add_task(&self, task: BotTask) {
    //     self.tasks.write().await.push(task);
    // }

    pub async fn add(&self, name: impl AsRef<str>, task: BotTaskType, max_restarts: i32) {
        self.tasks.write().await.push(BotTask::new(name, task, max_restarts));
    }

    pub async fn list(&self) {
        for task in self.tasks.read().await.iter() {
            log!("{}", &task.name)
        }
    }

    pub async fn run_tasks(&self) {
        let tasks = &*self.tasks.read().await;
        let futures_stream = stream::iter(tasks);
        futures_stream
            .for_each_concurrent(None, |task| async move {
                while task.task_status.read().await.max_restarts == -1
                    || task.task_status.read().await.max_restarts >= task.task_status.read().await.restart_status
                {
                    if task.task_status.read().await.restart_status > 0 {
                        log_debug!("RESTARTING {}", format!("{:?}", &task));
                    } else {
                        log_debug!("STARTING {}", format!("{:?}", &task));
                    }
                    let _ = task.run().await;
                    task.task_status.write().await.restart_status += 1;
                }
            })
            .await
    }

    pub async fn get_stats(&self) -> Vec<BotTaskStatus> {
        self.tasks
            .read()
            .await
            .iter()
            .map(|task| block_on(task.task_status.read()).clone())
            .collect::<Vec<BotTaskStatus>>()
    }
}
// Monitoring Task for all tasks helper async function to load as default
async fn task_monitor() -> Result<()> {
    let task_monitor_tick = tokio::time::interval(Duration::from_secs(TASK_MONITOR_TIME));
    pin_mut!(task_monitor_tick);

    loop {
        tokio::select! {
            _ = task_monitor_tick.tick() => {
                for task in TASKS_MANAGER.tasks.read().await.iter(){
                    log!("Task {:<20}: {}", task.name, task.task_status.read().await.get_stats().await)
                } ;
            }
        }
    }
}
