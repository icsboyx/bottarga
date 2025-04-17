#![allow(dead_code)]
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};

use eyre::Result;
use msedge_tts::tts::SpeechConfig;
use msedge_tts::voice::{Voice, get_voices_list};
use rand::Rng;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::CONFIG_DIR;
use crate::audio_player::TTS_AUDIO_QUEUE;
use crate::bot_commands::BOT_COMMANDS;
use crate::common::{MSGQueue, PersistentConfig};
use crate::irc_parser::IrcMessage;
use crate::twitch_client::{TWITCH_BOT_INFO, TWITCH_RECEIVER};
use crate::users::{USER_DB, USER_DEFAULT_VOICE_CONFIG};

pub static TTS_VOCE_BD: LazyLock<VoiceDB> = LazyLock::new(|| VoiceDB::default());
pub static TTS_QUEUE: LazyLock<MSGQueue<TTSMassage>> = LazyLock::new(|| MSGQueue::new());
static TRANSFORM_CHARS: &[(char, &str)] = &[('&', "and"), ('%', "percent")];

pub async fn start() -> Result<()> {
    // This is calling the warm_up method on the USER_DB, to preload all users
    USER_DB.read().await.warm_up();
    USER_DEFAULT_VOICE_CONFIG.warm_up();

    // This is saving the TTS_VOCE_BD to the CONFIG_DIR, for user consultation
    // Does not have real impact on the code.
    TTS_VOCE_BD.save(CONFIG_DIR).await;

    // Registering the list_locales and reset_voice commands
    BOT_COMMANDS
        .add_command(
            "list_locales",
            Arc::new(|irc_message| Box::pin(bot_cmd_tts_list_all_locales(irc_message))),
        )
        .await;

    // Registering the reset_voice command
    BOT_COMMANDS
        .add_command(
            "reset_voice",
            Arc::new(|irc_message| Box::pin(bot_cmd_tts_reset_voice(irc_message))),
        )
        .await;

    // This is the main loop for the TTS system, waiting for message.
    while let Some(tts_message) = TTS_QUEUE.next().await {
        text_to_speech(tts_message).await?;
    }
    Ok(())
}
#[derive(Debug, Clone)]
pub struct TTSMassage {
    pub speech_config: SpeechConfig,
    pub payload: String,
}

impl Default for TTSMassage {
    fn default() -> Self {
        Self {
            speech_config: SpeechConfig {
                voice_name: "".into(),
                audio_format: "".into(),
                pitch: 0,
                rate: 0,
                volume: 0,
            },
            payload: "".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceDB {
    voice_list: Vec<Voice>,
}

impl Default for VoiceDB {
    fn default() -> Self {
        Self {
            voice_list: get_voices_list().unwrap(),
        }
    }
}

impl PersistentConfig for VoiceDB {}

impl VoiceDB {
    pub async fn list_all_voices(&self) -> Vec<&String> {
        self.voice_list.iter().map(|v| &v.name).collect::<Vec<_>>()
    }

    pub fn filter_voices_by_text(&self, filter: &[&str]) -> Self {
        let voice_list = self
            .voice_list
            .iter()
            .cloned()
            .filter(|v| {
                let v_text = format! {"{:?}", v};
                filter
                    .iter()
                    .all(|f| v_text.to_lowercase().contains(f.to_lowercase().as_str()))
            })
            .collect::<Vec<_>>();

        if voice_list.is_empty() {
            log_debug!(
                "No voices found for filter: {:?}, no filter is applied",
                filter.as_ref()
            );
            return self.clone();
        }
        Self { voice_list }
    }

    pub async fn list_all_locales(&self) -> Vec<String> {
        let mut locales = HashSet::new();

        // Collect unique locales into the HashSet
        for voice in &self.voice_list {
            if let Some(locale) = &voice.locale {
                locales.insert(locale.clone()); // Insert clones to avoid ownership issues
            }
        }

        locales.into_iter().collect()
    }

    async fn filter_locale(&self, locale: impl AsRef<str>) -> Self {
        let locale = locale.as_ref().to_lowercase();

        let voice_list = self
            .voice_list
            .iter()
            .filter(|voice| {
                if let Some(v_locale) = &voice.locale {
                    v_locale.to_lowercase().contains(locale.as_str())
                } else {
                    false
                }
            })
            .cloned()
            .collect::<Vec<_>>();

        if voice_list.is_empty() {
            log_debug!("No voices found for locale: {}, no filter is applied", locale);
            return self.clone();
        }
        Self { voice_list }
    }

    async fn filter_gender(&self, gender: impl AsRef<str>) -> Self {
        let gender = gender.as_ref().to_lowercase();

        let voice_list = self
            .voice_list
            .iter()
            .filter(|voice| {
                if let Some(v_gender) = &voice.gender {
                    v_gender.to_lowercase().contains(gender.as_str())
                } else {
                    false
                }
            })
            .cloned()
            .collect::<Vec<_>>();

        if voice_list.is_empty() {
            log_debug!("No correct gender found: {}, no filter is applied", gender);
            return self.clone();
        }
        Self { voice_list }
    }

    pub fn random(&self) -> &Voice {
        let mut rng = rand::rng();
        let index = rng.random_range(0..self.voice_list.len());
        &self.voice_list[index]
    }
}
pub async fn text_to_speech(message: TTSMassage) -> Result<()> {
    let text = remove_url_in_text(message.payload);
    let text = text
        .chars()
        .map(|c| {
            TRANSFORM_CHARS
                .iter()
                .fold(c.to_string(), |acc, (char_to_replace, replacement)| {
                    acc.replace(*char_to_replace, replacement)
                })
        })
        .collect::<String>();

    let mut tts = msedge_tts::tts::client::connect_async().await?;
    let audio = tts.synthesize(text.as_ref(), &message.speech_config).await?;
    if audio.audio_bytes.is_empty() {
        return Ok(());
    }

    TTS_AUDIO_QUEUE.push_back(audio.audio_bytes).await;

    Ok(())
}

fn remove_url_in_text(text: impl AsRef<str>) -> String {
    // let url_regex = Regex::new(r"[a-zA-Z]+://[^\s]+").unwrap();
    let url_regex = Regex::new(r"(?:[a-zA-Z]+://|www\.|[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/)[^\s]+").unwrap();
    url_regex.replace_all(text.as_ref(), ", URL removed,").to_string()
}

pub async fn voice_msg(payload: &impl AsRef<str>, nick: &impl AsRef<str>) -> TTSMassage {
    let speech_config = if nick.as_ref() != TWITCH_BOT_INFO.nick_name().await {
        &USER_DB.write().await.get_user(nick).await.get_speech_config().clone()
    } else {
        TWITCH_BOT_INFO.speech_config().await
    };
    TTSMassage {
        speech_config: speech_config.clone(),
        payload: payload.as_ref().into(),
    }
}

pub async fn bot_cmd_tts_list_all_locales(_message: IrcMessage) -> Result<()> {
    let ret_val = format!("Available locales: {}", TTS_VOCE_BD.list_all_locales().await.join(", "));
    TWITCH_RECEIVER.send_privmsg(ret_val).await;
    Ok(())
}

pub async fn bot_cmd_tts_reset_voice(message: IrcMessage) -> Result<()> {
    let nick = message.sender;
    let filter = &message.payload.split_whitespace().collect::<Vec<_>>()[1..];
    USER_DB
        .write()
        .await
        .update_user(&nick, (TTS_VOCE_BD.filter_voices_by_text(filter).random()).into())
        .await;
    let payload = format!(
        "@{}, your voice config has been updated to {}",
        nick,
        USER_DB
            .write()
            .await
            .get_user(&nick)
            .await
            .get_speech_config()
            .voice_name
    );
    TTS_QUEUE
        .push_back(voice_msg(&payload, &TWITCH_BOT_INFO.nick_name().await).await)
        .await;
    TWITCH_RECEIVER.send_privmsg(payload).await;
    Ok(())
}
