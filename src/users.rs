use std::collections::HashMap;
use std::sync::LazyLock;

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

impl UsersDB {
    pub fn init(config_dir: Option<&str>) -> UsersDB {
        block_on(async { UsersDB::load(config_dir).await })
    }

    // This will be called on bot start to preload all users
    pub fn warm_up(&self) {}

    pub async fn add_new_user(&mut self, nick: impl AsRef<str>) -> User {
        let user = User::new(&nick);
        self.users.insert(nick.as_ref().into(), user.clone());
        (self).save(CONFIG_DIR).await;
        user
    }

    pub async fn update_user(&mut self, nick: impl AsRef<str>, speech_config: SpeechConfig) -> User {
        self.users.insert(nick.as_ref().into(), User {
            nick: nick.as_ref().into(),
            speech_config,
        });
        let _ = (*self).save(CONFIG_DIR).await;
        self.get_user(nick).await
    }

    // This will return if user exist in db or generate new user
    pub async fn get_user(&mut self, nick: impl AsRef<str>) -> User {
        if let Some(user) = self.users.get(nick.as_ref()) {
            user.clone()
        } else {
            log_debug!("User not found, creating new user: {}", nick.as_ref());
            self.add_new_user(nick).await
        }
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
            speech_config: TTS_VOCE_BD.filter_voices_by_text(&["it-IT"]).random().into(),
        }
    }

    pub fn get_speech_config(&self) -> &SpeechConfig {
        &self.speech_config
    }
}
