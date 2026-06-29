# Audio Player

The audio player consumes synthesized audio from `TTS_AUDIO_QUEUE` and plays it locally.

## Runtime Flow

```text
TTS_AUDIO_QUEUE
      |
      v
audio_player::start()
      |
      +--> Linux configured sink: PulseAudio playback
      |
      +--> default path: Kira playback
```

## Playback Control

`TTS_AUDIO_CONTROL` tracks playback state:

- `Ready`
- `Busy`
- `Stop`
- `Play`

The `!stop` command calls `bot_cmd_stop_audio`, which requests playback stop through `TTS_AUDIO_CONTROL`.

## Configuration

Audio settings are stored in `.config/AudioControl.toml`.

Important fields:

- `volume`: playback volume used by Kira.
- `linux_sink_name`: optional PulseAudio sink name. When set on Linux, Bottarga routes playback to that sink.

If `linux_sink_name` is not set, playback falls back to Kira.
