use std::collections::VecDeque;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

use eyre::Result;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, create_dir_all, metadata};
use tokio::sync::RwLock;

use crate::*;

pub(crate) trait PersistentConfig {
    async fn save<'a>(&self, config_dir: Option<&'a str>)
    where
        Self: Default + Serialize + for<'de> Deserialize<'de>,
    {
        let file_name = std::any::type_name::<Self>().split("::").last().unwrap().to_owned() + ".toml";
        let file_path = if config_dir.is_some() {
            PathBuf::from(std::env::current_dir().unwrap())
                .join(config_dir.unwrap())
                .join(&file_name)
        } else {
            PathBuf::from(std::env::current_dir().unwrap()).join(&file_name)
        };

        match self.check_file_path(&file_path).await {
            Ok(_) => log!("{} Config path checked successfully", &file_name),
            Err(e) => {
                log_error!("Unable to write to file: {:?}. Error: {}", file_path.parent(), e);
                log_warning!("Proceeding in-memory only, config will not be persistent.");
            } // Convert the error into Ok(()) , so the caller can proceed in memory only
        }
        match toml::to_string_pretty(&self) {
            Ok(ret_val) => match fs::write(&file_path, ret_val).await {
                Ok(_) => {
                    log!("{} Config saved successfully: {}", &file_name, file_path.display());
                }
                Err(e) => {
                    log_error!("Unable to write to file: {:?}. Error: {}", file_path.display(), e);
                    log_warning!("Proceeding in-memory only, config will not be persistent.");
                }
            },
            Err(e) => {
                log_error!("Unable to serialize config: {}. Error: {}", file_path.display(), e);
                log_warning!("Proceeding in-memory only, config will not be persistent.");
            }
        }
    }

    async fn check_file_path(&self, file_path: &PathBuf) -> Result<()> {
        if let Some(parent) = file_path.parent() {
            if !metadata(parent).await.is_ok() {
                log!("Parent directory does not exist, creating: {}", parent.display());
                match create_dir_all(parent).await {
                    Ok(_) => log!("Parent directory created successfully"),
                    Err(e) => {
                        log_error!("Failed to create parent directory: {}", e);
                        return Err(e.into());
                    }
                }
            }
        }
        Ok(())
    }

    async fn load(config_dir: Option<&str>) -> Self
    where
        Self: Default + Serialize + for<'de> Deserialize<'de>,
    {
        let file_name = std::any::type_name::<Self>().split("::").last().unwrap().to_owned() + ".toml";
        let file_path = if config_dir.is_some() {
            PathBuf::from(std::env::current_dir().unwrap())
                .join(config_dir.unwrap())
                .join(&file_name)
        } else {
            PathBuf::from(std::env::current_dir().unwrap()).join(&file_name)
        };

        match fs::read_to_string(&file_path).await {
            Ok(content) => {
                log!("{} config loaded successfully: {}", &file_name, file_path.display());
                match toml::from_str(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        log_error!("Unable to parse file: {}. Error: {}", file_path.display(), e.message());
                        log_warning!("Proceeding in-memory only, config will not be persistent.");
                        Self::default()
                    }
                }
            }
            Err(e) => {
                log_error!("Unable to read file: {}. Error: {}", file_path.display(), e);
                log_warning!("Trying to create default config.");
                let ret_val = Self::default();
                ret_val.save(config_dir).await;
                ret_val
            }
        }
    }
}

#[derive(Debug, Clone)]

pub struct BroadCastChannel<T>
where
    T: Sync + Send + Clone + 'static,
{
    broadcaster: tokio::sync::broadcast::Sender<T>,
}

impl<BM> BroadCastChannel<BM>
where
    BM: Sync + Send + Clone + Debug + 'static,
{
    pub fn new(capacity: usize) -> Self {
        let (broadcaster_tx, _) = tokio::sync::broadcast::channel(capacity);
        Self {
            broadcaster: broadcaster_tx,
        }
    }

    pub fn init(&self) -> &Self {
        self
    }

    pub async fn send_broadcast(&self, message: BM) -> Result<()> {
        if self.broadcaster.receiver_count() > 0 {
            self.broadcaster.send(message)?;
        }
        Ok(())
    }

    pub async fn subscribe_broadcast(&self) -> tokio::sync::broadcast::Receiver<BM> {
        self.broadcaster.subscribe()
    }
}

#[derive(Debug)]
pub struct MSGQueue<T>
where
    T: Sync + Send + Clone + Debug + 'static,
{
    queue: RwLock<VecDeque<T>>,
    notify: Arc<tokio::sync::Notify>,
}

impl<T> MSGQueue<T>
where
    T: Sync + Send + Clone + Debug + 'static,
{
    pub fn new() -> Self {
        Self {
            queue: RwLock::new(VecDeque::new()),
            notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    pub async fn push_back(&self, payload: T) {
        self.queue.write().await.push_back(payload);
        self.notify.notify_waiters();
    }

    pub async fn next(&self) -> Option<T> {
        loop {
            if let Some(value) = self.queue.write().await.pop_front() {
                return Some(value);
            }
            self.notify.notified().await;
        }
    }

    pub async fn next_error(&self) -> Result<T> {
        loop {
            if let Some(value) = self.queue.write().await.pop_front() {
                return Ok(value);
            }
            self.notify.notified().await;
        }
    }

    pub async fn len(&self) -> usize {
        self.queue.read().await.len()
    }
}
