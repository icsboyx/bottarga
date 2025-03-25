# Audio Player Module Documentation

This document provides an overview of the `audio_player.rs` module, which is responsible for handling audio playback functionality in the application.

## Overview

The `audio_player.rs` module provides tools for managing audio playback, including playing, stopping, and controlling audio streams. It supports multiple backends, such as `kira` and `pulse`, and is designed to work asynchronously using `tokio`.

## Key Components

### 1. **Enums**

- **`PlayerCommands`**: Represents the state of the audio player. Possible values:
  - `Play`
  - `Stop`
  - `Ready`
  - `Busy`

### 2. **Static Variables**

- **`TTS_AUDIO_QUEUE`**: A queue for storing audio data to be played.
- **`TTS_AUDIO_CONTROL`**: Manages the playback state and control flow.

### 3. **Structs**

- **`AudioPlayControl`**: Provides methods to control the playback state (`play`, `stop`, `busy`, `ready`, etc.).

### 4. **Functions**

- **`start`**: Initializes the audio player and listens for audio data in the queue.
- **`play_on_bot`**: Plays audio using the PulseAudio backend (Linux only).
- **`play_on_kira`**: Plays audio using the `kira` audio library.
- **`stop_audio`**: Stops the currently playing audio.
  - **Bot Command**: The `stop` command can be triggered via `BOT_COMMANDS` to stop audio playback remotely.

## Platform-Specific Features

- The module includes functionality specific to Linux, such as PulseAudio integration, which is conditionally compiled using `#[cfg(target_os = "linux")]`.

## Dependencies

- **`kira`**: For audio playback management.
- **`rodio`**: For decoding audio streams.
- **`pulse`**: For PulseAudio integration (Linux only).
- **`tokio`**: For asynchronous operations.

## Usage

To use this module, ensure the required dependencies are included in your `Cargo.toml` file. The `start` function should be called to initialize the audio player, and audio data can be added to the `TTS_AUDIO_QUEUE` for playback.
