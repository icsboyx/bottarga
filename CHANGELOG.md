# Changelog
All notable changes to this project will be documented in this file.

## [0.3.2] - 2026-11-05

### Fixed
- Restored text-to-speech synthesis, which failed with `403 Forbidden` after Microsoft made the `Sec-MS-GEC` v2 token mandatory in October 2026. The fix comes from `msedge-tts` 0.4.1 ([changelog](https://github.com/hs-CN/msedge-tts/blob/master/CHANGELOG.md)).

### Changed
- Replaced the forked `msedge-tts` git dependency with the upstream crates.io release `0.4.1`, using the `tokio-runtime` feature.
- Updated the TTS client connection to the `tokio_runtime` API.
- Updated Rust dependencies, including `tokio-tungstenite` 0.30, `toml` 1.2, `rodio` 0.23, `kira` 0.12 and `rand` 0.10, and adapted audio playback to the new `rodio` API.
- Simplified time formatting in the `now!` macro using `chrono`.

## [0.3.1] - 2026-09-13

### Changed
- Refreshed Rust dependencies to current compatible versions.
- Clarified the release line after the migration from the old IRC chat path to Twitch EventSub WebSocket.

### Release Notes
- Bottarga's main branch now represents the WebSocket-based Twitch chat implementation. The project connects through Twitch EventSub WebSocket, receives chat events, replies through Helix, and keeps the release build pipeline ready for packaged Linux and Windows binaries.

## [0.3.0] - 2026-09-13

### Added
- GitHub Actions release workflow for Linux and Windows binaries.
- README release instructions for publishing tagged builds.

### Changed
- Bumped package version to `0.3.0`.
- Cleaned up Clippy warnings so strict lint checks pass.
- Refreshed package description.

## [0.1.1] - 2026-01-09

### Added
- Missing import for rodio::Decoder and Source on Linux

### Changed
- Updated msedge-tts dependency to use forked version with updated USER_AGENT for better platform compatibility ([ed2cc3c](https://github.com/icsboyx/msedge-tts/commit/ed2cc3c87772f66329221975ef6bb8e14c96b641))

### Fixed
- 

## [0.1.0] - 2026-01-08

### Added
- Initial release of the Bottarga project.
- Core functionality for building and running the Rust application.
- Release workflow for CI automation.

### Changed
- Optimized build process using `cargo build --release`.
- Updated documentation to include setup and usage instructions.
