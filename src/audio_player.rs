use std::fs::OpenOptions;
use std::io::{Cursor, Write};
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
use rodio::{Decoder, Source};
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
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

    pub async fn set_status_play(&self) {
        *self.status.write().await = PlayerCommands::Play;
        self.notify.notify_one();
    }

    pub async fn set_status_stop(&self) {
        if self.get_status().await != PlayerCommands::Busy {
            log_debug!("Audio is not playing");
            return;
        }
        *self.status.write().await = PlayerCommands::Stop;
        self.notify.notify_one();
    }

    pub async fn set_status_busy(&self) {
        log_trace!("Audio player set to busy");
        *self.status.write().await = PlayerCommands::Busy
    }

    pub async fn set_status_ready(&self) {
        *self.status.write().await = PlayerCommands::Ready
    }

    pub async fn get_event(&self) -> PlayerCommands {
        self.notify.notified().await;
        let event = self.status.read().await.clone();
        event
    }

    pub async fn get_status(&self) -> PlayerCommands {
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

    TTS_AUDIO_CONTROL.set_status_busy().await;
    log_debug!("Setting audio player to busy");
    let audio_chunks = audio.chunks(1024);
    TTS_AUDIO_CONTROL.set_status_busy().await;
    for chunk in audio_chunks {
        match TTS_AUDIO_CONTROL.get_status().await {
            PlayerCommands::Busy => {
                sink.write(chunk)?;
            }
            PlayerCommands::Stop => {
                log_debug!("Stopping audio playback");
                TTS_AUDIO_CONTROL.set_status_ready().await;
                log_debug!("Setting audio player ready");
                break;
            }
            _ => {}
        }
        // if TTS_AUDIO_CONTROL.get_status().await != PlayerCommands::Stop {
        //     sink.write(chunk)?;
        // } else {
        //     log_debug!("Stopping audio playback");
        //     TTS_AUDIO_CONTROL.set_status_ready().await;
        //     log_debug!("Setting audio player ready");
        //     break;
        // }
    }
    log_debug!("Audio playback finished");
    TTS_AUDIO_CONTROL.set_status_ready().await;

    Ok(())
}

pub async fn bot_cmd_stop_audio(_message: IrcMessage) -> Result<()> {
    TTS_AUDIO_CONTROL.set_status_stop().await;
    Ok(())
}

pub async fn play_on_kira(audio: Vec<u8>) -> Result<()> {
    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
    let sound_data = StaticSoundData::from_cursor(Cursor::new(audio))?.volume(AUDIO_CONTROL.volume);
    let sound = manager.play(sound_data.clone())?;

    TTS_AUDIO_CONTROL.set_status_busy().await;
    while TTS_AUDIO_CONTROL.get_status().await != PlayerCommands::Stop
        && sound.state() == kira::sound::PlaybackState::Playing
    {
        sleep(Duration::from_millis(100));
    }
    TTS_AUDIO_CONTROL.set_status_ready().await;
    Ok(())
}

// ############################
// pub async fn play_on_pipe(audio: Vec<u8>, pipe: impl AsRef<str>) -> Result<()> {
//     // let cursor = Cursor::new(audio);
//     // let source = Decoder::new(cursor)?.convert_samples::<u16>();
//     // let sample_rate = source.sample_rate();
//     // let channels = source.channels() as u8;

//     let mut pipe = OpenOptions::new().write(true).open("/tmp/audio_pipe.mp3")?;
//     // log_trace!("Sample rate: {}, channels {}.", sample_rate, channels);
//     // log_trace!("Playing on pipe: {:?}", pipe.metadata());

//     // let audio_data = source.into_iter().collect::<Vec<_>>();

//     log_debug!("Setting audio player to busy");
//     let audio_chunks = audio.chunks(1024);
//     TTS_AUDIO_CONTROL.set_status_busy().await;

//     for chunk in audio_chunks {
//         match TTS_AUDIO_CONTROL.get_status().await {
//             PlayerCommands::Busy => {
//                 pipe.write(&chunk)?;
//                 sleep(Duration::from_millis(10));
//             }
//             PlayerCommands::Stop => {
//                 log_debug!("Stopping audio playback");
//                 TTS_AUDIO_CONTROL.set_status_ready().await;
//                 log_debug!("Setting audio player ready");
//                 break;
//             }
//             _ => {}
//         }
//     }
//     log_debug!("Audio playback finished");
//     TTS_AUDIO_CONTROL.set_status_ready().await;

//     Ok(())
// }

// pub async fn play_udp(audio: Vec<u8>, _sink: impl AsRef<str>) -> Result<()> {
//     let udp_socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
//     udp_socket.connect("127.0.0.1:12345").await?;

//     let cursor = Cursor::new(audio);
//     let source = Decoder::new(cursor)?.convert_samples::<f32>();
//     let sample_rate = source.sample_rate();
//     let channels = source.channels() as u8;

//     log_trace!("Sample rate: {}, channels {}.", sample_rate, channels);

//     // let (_stream, stream_handle) = OutputStream::try_default().unwrap();

//     let spec = Spec {
//         format: Format::FLOAT32NE,
//         channels,
//         rate: sample_rate,
//     };
//     assert!(spec.is_valid());
//     let audio_data = source.into_iter().collect::<Vec<_>>();
//     // let audio_chunks = source.chunks(1024);

//     let audio_bytes = audio_data
//         .iter()
//         .flat_map(|&x| x.to_le_bytes().to_vec())
//         .collect::<Vec<_>>();

//     let audio_chunks = audio_bytes.chunks(1024);
//     // Calculate delay for each chunk (based on sample rate, channels, and chunk size)
//     let chunk_size = 1024; // Size of each chunk in bytes
//     let samples_per_chunk = chunk_size / std::mem::size_of::<f32>();
//     let seconds_per_chunk = samples_per_chunk as f64 / (sample_rate as f64 * channels as f64);
//     let delay = Duration::from_secs_f64(seconds_per_chunk);

//     TTS_AUDIO_CONTROL.set_status_busy().await;
//     log_debug!("Setting audio player to busy");
//     for chunk in audio_chunks {
//         match TTS_AUDIO_CONTROL.get_status().await {
//             PlayerCommands::Busy => {
//                 tokio::time::sleep(delay).await;
//             }
//             PlayerCommands::Stop => {
//                 log_debug!("Stopping audio playback");
//                 TTS_AUDIO_CONTROL.set_status_ready().await;
//                 log_debug!("Setting audio player ready");
//                 break;
//             }
//             _ => {}
//         }
//     }
//     log_debug!("Audio playback finished");
//     TTS_AUDIO_CONTROL.set_status_ready().await;
//     Ok(())
// }

// pub async fn play_udp(audio: Vec<u8>, _sink: impl AsRef<str>) -> Result<()> {
//     let udp_socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
//     udp_socket.connect("127.0.0.1:12345").await?;

//     udp_socket.send(&audio).await?;

//     return Ok(());
//     let cursor = Cursor::new(audio);
//     let source = Decoder::new(cursor)?.convert_samples::<i16>();

//     let sample_rate = source.sample_rate();
//     let channels = source.channels() as u8;

//     log_debug!("Audio chunks duration: {:?}", source.inner().total_duration());

//     let audio_chunks = source.into_iter().collect::<Vec<_>>();
//     log_debug!("Sample rate: {}, channels {}.", sample_rate, channels);

//     let audio_duration = audio_chunks.len() as u32 / (sample_rate * channels as u32);
//     println!("Total chunk count: {}", audio_chunks.len() / channels as usize);
//     println!("Audio duration: {:?}", audio_duration);
//     println!(
//         "duration of single chunk is {:?}",
//         audio_duration as f32 / audio_chunks.len() as f32
//     );

//     let spec = Spec {
//         format: Format::F32le, // You can safely send this as f32le via `to_le_bytes()`
//         channels,
//         rate: sample_rate,
//     };

//     TTS_AUDIO_CONTROL.set_status_busy().await;
//     for chunk in audio_chunks {
//         match TTS_AUDIO_CONTROL.get_status().await {
//             PlayerCommands::Busy => {
//                 // TODO: send chunk (e.g. over UDP, to pipe, etc.)
//                 // Convert f32 samples to bytes
//                 let bytes = chunk.to_le_bytes();

//                 //Example: send via socket or write to pipe
//                 udp_socket.send(&bytes).await?;

//                 // tokio::time::sleep(chunk_duration).await;
//             }
//             PlayerCommands::Stop => {
//                 TTS_AUDIO_CONTROL.set_status_ready().await;
//                 break;
//             }
//             _ => {}
//         }
//     }

//     TTS_AUDIO_CONTROL.set_status_ready().await;
//     Ok(())
// }
