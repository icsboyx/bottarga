use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, LazyLock};

use anyhow::{Error, Result};
use colored::{Color, Colorize};
use futures::stream;
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::sync::RwLock;

pub static TASKS_MANAGER: LazyLock<TaskManager> = LazyLock::new(|| TaskManager::default());

// type BotTaskType = dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>>;
type BotTaskType = fn() -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>>;

#[derive(Debug)]
pub struct BotTask {
    name: String,
    function: RwLock<BotTaskType>,
    max_restarts: RwLock<i32>,
    restart_status: RwLock<i32>,
    color: Color,
}

impl BotTask {
    pub fn new(name: impl AsRef<str>, task: BotTaskType, max_restarts: i32, color: Color) -> Self {
        Self {
            max_restarts: RwLock::new(max_restarts),
            name: name.as_ref().into(),
            function: RwLock::new(task),
            restart_status: RwLock::new(0),
            color,
        }
    }

    pub async fn run(&self) -> Result<()> {
        (self.function.read().await)().await
    }
}

#[derive(Default, Debug)]
pub struct TaskManager {
    pub tasks: RwLock<FuturesUnordered<BotTask>>,
}

impl TaskManager {
    pub async fn add_task(&self, task: BotTask) {
        self.tasks.write().await.push(task);
    }

    pub async fn add(&self, name: impl AsRef<str>, task: BotTaskType, max_restarts: i32, color: Color) {
        self.tasks
            .write()
            .await
            .push(BotTask::new(name, task, max_restarts, color));
    }

    pub async fn list(&self) {
        for task in self.tasks.read().await.iter() {
            println!("{:?}", &task.name)
        }
    }

    pub async fn runn_tasks(&self) {
        let tasks = &*self.tasks.read().await;
        let futures_stream = stream::iter(tasks);
        futures_stream
            .for_each_concurrent(None, |task| async move {
                while *task.max_restarts.read().await >= *task.restart_status.read().await {
                    if *task.restart_status.read().await > 0 {
                        println!("RESTARTING {}", format!("{:#?}", &task).color(task.color));
                    } else {
                        println!("STARTING {}", format!("{:#?}", &task).color(task.color));
                    }
                    let _ = task.run().await;
                    println!("{}", format!("task {} terminated", &task.name).color(task.color));
                    *task.restart_status.write().await += 1;
                }
            })
            .await
    }

    // pub async fn runn_tasks(&self) {
    //     let futures_stream = stream::iter(&self.tasks);
    //     futures_stream
    //         .for_each_concurrent(None, |task| async move {
    //             while *task.max_restarts.read().await >= *task.restart_status.read().await {
    //                 if *task.restart_status.read().await > 0 {
    //                     println!("RESTARTING {}", format!("{:#?}", &task).color(task.color));
    //                 } else {
    //                     println!("STARTING {}", format!("{:#?}", &task).color(task.color));
    //                 }
    //                 let _ = task.run().await;
    //                 println!("{}", format!("task {} terminated", &task.name).color(task.color));
    //                 *task.restart_status.write().await += 1;
    //             }
    //         })
    //         .await
    // }

    pub async fn print_stats(&self) {
        for task in self.tasks.read().await.iter() {
            println!("{}", format!("{:?}", task).color(task.color))
        }
    }
}

// #[derive(Default)]
// pub struct Tasksmanager {
//     pub tasks: Vec<BotTask>,
// }

// impl Tasksmanager {
//     pub fn add_task(&mut self, task: BotTask) {
//         self.tasks.push(task);
//     }

//     pub async fn runn_tasks(self) -> Vec<JoinHandle<Result<(), Error>>> {
//         let mut jh = vec![];
//         for task in self.tasks {
//             jh.push(tokio::spawn(async move { task.function.await }));
//         }
//         jh
//     }

//     pub fn list(&self) {
//         for task in self.tasks.iter() {
//             println!("{:}", task.name)
//         }
//     }
// }
