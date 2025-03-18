use std::io::Cursor;
use std::sync::LazyLock;

use anyhow::Result;
use psimple::Simple;
use pulse::sample::{Format, Spec};
use pulse::stream::Direction;
use rodio::{Decoder, OutputStream};
use tokio::sync::RwLock;

use crate::bot_commands::BOT_COMMANDS;
use crate::defs::MSGQueue;
use crate::irc_parser::IrcMessage;

pub static TTS_AUDIO_QUEUE: LazyLock<MSGQueue<Vec<u8>>> = LazyLock::new(|| MSGQueue::new());
pub static TTS_AUDIO_STOP: LazyLock<RwLock<bool>> = LazyLock::new(|| RwLock::new(false));

pub async fn start() -> Result<()> {
    BOT_COMMANDS
        .add_command("stop", |irc_message| Box::pin(stop_audio(irc_message)))
        .await;

    while let Some(audio) = TTS_AUDIO_QUEUE.next().await {
        play_audio(audio).await?;
    }
    Ok(())
}

pub async fn play_audio(audio: Vec<u8>) -> Result<()> {
    use std::io::Cursor;

    use rodio::{Decoder, Sink};

    let cursor = Cursor::new(audio);
    let source = Decoder::new(cursor)?;

    let (_, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(source);
    // Wait until end or TTS_AUDIO_STOP is true
    while sink.empty() && !*TTS_AUDIO_STOP.read().await {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}

pub async fn play_on_bot(audio: Vec<u8>) -> Result<()> {
    let cursor = Cursor::new(audio);
    let source = Decoder::new(cursor)?;

    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let spec = Spec {
        format: Format::S16le,
        channels: 1,
        rate: 24000,
    };
    assert!(spec.is_valid());

    let sink = Simple::new(
        None,                // Use the default server
        "botox",             // Our application’s name
        Direction::Playback, // We want a playback stream
        Some("BOT.capture"), // Use the default device if failed
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

    sink.write(&audio).unwrap();
    sink.drain().unwrap();

    Ok(())
}

pub async fn stop_audio(_message: IrcMessage) -> Result<()> {
    *TTS_AUDIO_STOP.write().await = true;
    Ok(())
}
