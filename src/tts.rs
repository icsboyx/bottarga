#![allow(dead_code)]
use std::collections::HashSet;
use std::sync::LazyLock;

use anyhow::Result;
use msedge_tts::tts::SpeechConfig;
use msedge_tts::voice::{Voice, get_voices_list};
use rand::Rng;
use regex::Regex;

use crate::audio_player::TTS_AUDIO_QUEUE;
use crate::defs::MSGQueue;
use crate::twitch_client::TWITCH_BROADCAST;
use crate::users::USER_DB;

pub static TTS_VOCE_BD: LazyLock<VoiceDB> = LazyLock::new(|| VoiceDB::default());
pub static TTS_QUEUE: LazyLock<MSGQueue<String>> = LazyLock::new(|| MSGQueue::default());
static TRANSFORM_CHARS: &[(char, &str)] = &[('&', "and"), ('%', "percent")];

pub async fn start() -> Result<()> {
    let mut twitch_broadcast = TWITCH_BROADCAST.subscribe_broadcast().await;

    loop {
        tokio::select! {

            Ok(message) = twitch_broadcast.recv() => {
              text_to_speech(message.payload, USER_DB.write().await.get_user(message.sender).await.unwrap().get_speech_config()).await?;
          }

          Some(ret_val) = TTS_QUEUE.next() => {
            text_to_speech(ret_val, USER_DB.write().await.get_user("BOT").await.unwrap().get_speech_config()).await?;
          }
        }
    }
}

#[derive(Debug, Clone)]
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

impl VoiceDB {
    pub async fn list_all_voices(&self) -> Vec<&String> {
        self.voice_list.iter().map(|v| &v.name).collect::<Vec<_>>()
    }

    pub async fn filter_voices_by_text(&self, filter: &[&str]) -> Self {
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

    pub fn as_speech_config(&self) {}
}
pub async fn text_to_speech(text: impl AsRef<str>, speech_config: &SpeechConfig) -> Result<()> {
    let text = remove_url_in_text(text);
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
    let audio = tts.synthesize(text.as_ref(), speech_config).await?;
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
