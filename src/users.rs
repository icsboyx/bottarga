use std::collections::HashMap;
use std::sync::LazyLock;

use anyhow::Result;
use futures::executor::block_on;
use msedge_tts::tts::SpeechConfig;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::CONFIG_DIR;
use crate::defs::PersistentConfig;
use crate::tts::TTS_VOCE_BD;

pub static USER_DB: LazyLock<RwLock<UsersDB>> = LazyLock::new(|| RwLock::new(UsersDB::init(CONFIG_DIR)));

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UsersDB {
    users: HashMap<String, User>,
}

impl PersistentConfig for UsersDB {}
impl PersistentConfig for &mut UsersDB {}

impl UsersDB {
    pub fn init(config_dir: Option<&str>) -> UsersDB {
        block_on(UsersDB::load(config_dir))
    }

    pub async fn add_user(&mut self, nick: impl AsRef<str>) {
        self.users.insert(nick.as_ref().into(), User::new(nick));
    }

    // This will return if user exist in db or generate new user
    pub async fn get_user(&mut self, nick: impl AsRef<str>) -> Result<&User> {
        if self.users.contains_key(nick.as_ref()) {
            Ok(self.users.get(nick.as_ref()).unwrap())
        } else {
            self.add_user(nick.as_ref()).await;
            Ok(self.users.get(nick.as_ref()).unwrap())
        }
    }

    pub async fn update_user(&mut self, nick: impl AsRef<str>, speech_config: &SpeechConfig) {
        self.users.get_mut(nick.as_ref()).unwrap().speech_config = speech_config.clone();
        let _ = (*self).save(CONFIG_DIR).await;
    }
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    nick: String,
    speech_config: SpeechConfig,
}

impl Default for User {
    fn default() -> Self {
        Self {
            nick: "default".into(),
            speech_config: TTS_VOCE_BD.random().into(),
        }
    }
}

impl User {
    pub fn new(nick: impl AsRef<str>) -> Self {
        Self {
            nick: nick.as_ref().into(),
            speech_config: TTS_VOCE_BD.random().into(),
        }
    }

    pub fn get_speech_config(&self) -> &SpeechConfig {
        &self.speech_config
    }
}
