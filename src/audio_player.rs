use std::io::Cursor;
use std::sync::{Arc, LazyLock};
use std::thread::sleep;
use std::time::Duration;

use eyre::Result;
use futures::executor::block_on;
use kira::sound::static_sound::StaticSoundData;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend};
// compile this only for linux
#[cfg(target_os = "linux")]
use psimple::Simple;
// compile this only for linux
#[cfg(target_os = "linux")]
use pulse::sample::{Format, Spec};
// compile this only for linux
#[cfg(target_os = "linux")]
use pulse::stream::Direction;
use rodio::Decoder;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::CONFIG_DIR;
use crate::bot_commands::BOT_COMMANDS;
use crate::common::{MSGQueue, PersistentConfig};
use crate::irc_parser::IrcMessage;

pub static TTS_AUDIO_QUEUE: LazyLock<MSGQueue<Vec<u8>>> = LazyLock::new(|| MSGQueue::new());
pub static TTS_AUDIO_CONTROL: LazyLock<AudioPlayControl> = LazyLock::new(|| AudioPlayControl::new());
pub static AUDIO_CONTROL: LazyLock<AudioControl> = LazyLock::new(|| AudioControl::init(CONFIG_DIR));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioControl {
    volume: f32,
    linux_sink: Option<String>,
}

impl Default for AudioControl {
    fn default() -> Self {
        Self {
            volume: -6.0,
            linux_sink: None,
        }
    }
}

impl AudioControl {
    pub fn init(config_dir: Option<&str>) -> Self {
        block_on(AudioControl::load(config_dir))
    }

    pub fn warm_up(&self) {}
}

impl PersistentConfig for AudioControl {}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerCommands {
    Play,
    Stop,
    Ready,
    Busy,
}

pub struct AudioPlayControl {
    status: Arc<RwLock<PlayerCommands>>,
    notify: Arc<tokio::sync::Notify>,
}

impl AudioPlayControl {
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(PlayerCommands::Ready)),
            notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    pub async fn play(&self) {
        *self.status.write().await = PlayerCommands::Play;
        self.notify.notify_one();
    }

    pub async fn stop(&self) {
        if self.status().await != PlayerCommands::Busy {
            log_debug!("Audio is not playing");
            return;
        }
        *self.status.write().await = PlayerCommands::Stop;
        self.notify.notify_one();
    }

    pub async fn busy(&self) {
        *self.status.write().await = PlayerCommands::Busy
    }

    pub async fn event(&self) -> PlayerCommands {
        self.notify.notified().await;
        let event = self.status.read().await.clone();
        event
    }

    pub async fn ready(&self) {
        *self.status.write().await = PlayerCommands::Ready
    }

    pub async fn status(&self) -> PlayerCommands {
        self.status.read().await.clone()
    }
}

pub async fn start() -> Result<()> {
    // Warm up the AUDIO_CONTROL
    AUDIO_CONTROL.warm_up();
    BOT_COMMANDS
        .add_command(
            "stop",
            Arc::new(|irc_message| Box::pin(bot_cmd_stop_audio(irc_message))),
        )
        .await;

    while let Some(audio) = TTS_AUDIO_QUEUE.next().await {
        #[cfg(target_os = "linux")]
        if let Some(sink) = &AUDIO_CONTROL.linux_sink {
            match tokio::spawn(play_on_sink(audio.clone(), sink)).await {
                Ok(Ok(())) => {
                    log_debug!("Audio played on sink: {}", sink);
                }
                Ok(Err(e)) => {
                    log_error!("Error playing audio on sink: {}", e);
                }
                Err(e) => {
                    log_error!("Error playing audio on sink: {}", e);
                }
            }
        } else {
            tokio::spawn(play_on_kira(audio.clone())).await??;
        }
        #[cfg(not(target_os = "linux"))]
        tokio::spawn(play_on_kira(audio)).await??;
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn play_on_sink(audio: Vec<u8>, sink: impl AsRef<str>) -> Result<()> {
    use std::sync::Arc;

    use rodio::Source;

    let cursor = Cursor::new(audio);
    let source = Decoder::new(cursor)?.convert_samples::<f32>();
    let sample_rate = source.sample_rate();
    let channels = source.channels() as u8;

    log_trace!("Sample rate: {}, channels {}.", sample_rate, channels);

    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let spec = Spec {
        format: Format::FLOAT32NE,
        channels,
        rate: sample_rate,
    };
    assert!(spec.is_valid());

    let sink = Simple::new(
        None,                // Use the default server
        "botox",             // Our application’s name
        Direction::Playback, // We want a playback stream
        Some(sink.as_ref()), // Use the default device if failed
        "botox tts",         // Description of our stream
        &spec,               // Our sample format
        None,                // Use default channel map
        None,                // Use default buffering attributes
    )
    .unwrap();

    let audio_data = source.into_iter().collect::<Vec<_>>();
    let audio = audio_data
        .iter()
        .flat_map(|&x| x.to_le_bytes().to_vec())
        .collect::<Vec<_>>();

    let control_sink = Arc::new(sink).clone();
    let play_sink = control_sink.clone();
    TTS_AUDIO_CONTROL.busy().await;
    log_debug!("Setting audio player to busy");
    let audio_chunks = audio.chunks(1024);
    for chunk in audio_chunks {
        if *TTS_AUDIO_CONTROL.status.read().await != PlayerCommands::Stop {
            play_sink.write(chunk)?;
        } else {
            log_debug!("Stopping audio playback");
            TTS_AUDIO_CONTROL.ready().await;
            log_debug!("Setting audio player ready");
            break;
        }
    }

    Ok(())
}

pub async fn bot_cmd_stop_audio(_message: IrcMessage) -> Result<()> {
    TTS_AUDIO_CONTROL.stop().await;
    Ok(())
}

pub async fn play_on_kira(audio: Vec<u8>) -> Result<()> {
    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
    let sound_data = StaticSoundData::from_cursor(Cursor::new(audio))?.volume(AUDIO_CONTROL.volume);
    let sound = manager.play(sound_data.clone())?;

    TTS_AUDIO_CONTROL.busy().await;
    while TTS_AUDIO_CONTROL.status().await != PlayerCommands::Stop
        && sound.state() == kira::sound::PlaybackState::Playing
    {
        sleep(Duration::from_millis(100));
    }
    TTS_AUDIO_CONTROL.ready().await;
    Ok(())
}
