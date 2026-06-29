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
use crate::twitch_client::tw_api::send_chat_message;
use crate::twitch_client::tw_client::TwitchChatMessage;
use crate::twitch_client::tw_oauth_token::TW_TOKEN;
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
            Arc::new(|chat_message| Box::pin(bot_cmd_tts_list_all_locales(chat_message))),
        )
        .await;

    // Registering the reset_voice command
    BOT_COMMANDS
        .add_command(
            "reset_voice",
            Arc::new(|chat_message| Box::pin(bot_cmd_tts_reset_voice(chat_message))),
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
    let speech_config = if nick.as_ref() != TW_TOKEN.login().await {
        USER_DB.write().await.get_user(nick).await.get_speech_config().clone()
    } else {
        TTS_VOCE_BD.filter_voices_by_text(&["it-IT", "multi"]).random().into()
    };
    TTSMassage {
        speech_config,
        payload: payload.as_ref().into(),
    }
}

pub async fn bot_cmd_tts_list_all_locales(message: TwitchChatMessage) -> Result<()> {
    let ret_val = format!("Available locales: {}", TTS_VOCE_BD.list_all_locales().await.join(", "));
    message.reply(ret_val).await?;
    Ok(())
}

pub async fn bot_cmd_tts_reset_voice(message: TwitchChatMessage) -> Result<()> {
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
        .push_back(voice_msg(&payload, &TW_TOKEN.login().await).await)
        .await;
    send_chat_message(payload, Some(&message.message_id)).await?;
    Ok(())
}
