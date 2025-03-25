# TTS Module Documentation

This document provides an overview of the `tts.rs` file, which implements the Text-to-Speech (TTS) functionality for the application.

## Overview

The TTS module is responsible for managing text-to-speech operations, including voice configuration, message queuing, and integration with Twitch commands. It uses the `msedge_tts` library for speech synthesis and provides various utilities for filtering and managing voices.

---

## Key Components

### 1. **Static Variables**

- `TTS_VOCE_BD`: A lazily initialized `VoiceDB` instance that holds the list of available voices.
- `TTS_QUEUE`: A lazily initialized `MSGQueue` for queuing TTS messages.
- `TRANSFORM_CHARS`: A static array of character transformations (e.g., replacing `&` with "and").

---

### 2. **Structures**

#### `TTSMassage`

Represents a TTS message with the following fields:

- `speech_config`: Configuration for speech synthesis (e.g., voice name, pitch, rate).
- `payload`: The text to be converted to speech.

#### `VoiceDB`

Manages the list of available voices and provides methods for filtering and listing voices.

---

### 3. **Functions**

#### `start()`

- Initializes the TTS system, preloads user data, and registers commands (`list_locales` and `reset_voice`).
- Runs the main loop to process TTS messages from the queue.

#### `text_to_speech(message: TTSMassage) -> Result<()>`

- Converts the given text payload into speech audio using the specified `SpeechConfig`.
- Removes URLs from the text and applies character transformations.

#### `remove_url_in_text(text: impl AsRef<str>) -> String`

- Removes URLs from the input text using a regular expression.

#### `voice_msg(payload: &impl AsRef<str>, nick: &impl AsRef<str>) -> TTSMassage`

- Creates a `TTSMassage` for a given payload and user nickname.

#### `tts_list_all_locales(_message: IrcMessage) -> Result<()>`

- Lists all available locales and sends the result as a Twitch message.

#### `tts_reset_voice(message: IrcMessage) -> Result<()>`

- Resets the voice configuration for a user based on provided filters.

---

### 4. **VoiceDB Methods**

#### `list_all_voices() -> Vec<&String>`

- Returns a list of all available voice names.

#### `filter_voices_by_text(filter: &[&str]) -> Self`

- Filters voices based on a list of text filters.

#### `list_all_locales() -> Vec<String>`

- Returns a list of unique locales from the available voices.

#### `filter_locale(locale: impl AsRef<str>) -> Self`

- Filters voices by locale.

#### `filter_gender(gender: impl AsRef<str>) -> Self`

- Filters voices by gender.

#### `random() -> &Voice`

- Returns a random voice from the list.

---

## Command Registration

The following commands are registered for Twitch integration:

- **`list_locales`**:  
  Lists all available locales.  
  **Usage**: `!list_locales`  
  **Description**: Sends a message listing all available locales for TTS voices.

- **`reset_voice`**:  
  Resets the voice configuration for a user based on provided filters.  
  **Usage**: `!reset_voice <filter>`  
  **Description**: Updates the user's voice configuration to a random voice matching the specified filter.

---

## Main Loop

The `start()` function contains the main loop, which processes messages from the `TTS_QUEUE` and converts them to speech.

---

## Dependencies

- `msedge_tts`: For speech synthesis.
- `regex`: For URL removal.
- `rand`: For random voice selection.
- `serde`: For serialization and deserialization of voice data.

---

## Notes

- The `PersistentConfig` trait is implemented for `VoiceDB` to allow saving and loading voice configurations.
- The `remove_url_in_text` function ensures that URLs are sanitized before speech synthesis.

---

## Future Improvements

- Add more robust error handling for voice filtering and synthesis.
- Optimize the main loop for better performance under high message loads.
