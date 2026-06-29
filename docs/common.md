# Common Utilities

`common.rs` contains shared runtime primitives used by multiple modules.

## PersistentConfig

`PersistentConfig` saves and loads TOML files under the configured directory.

The filename is derived from the Rust type name. For example:

- `TwitchToken` -> `TwitchToken.toml`
- `StreamerChannel` -> `StreamerChannel.toml`
- `UsersDB` -> `UsersDB.toml`

If a file cannot be read or parsed, the default value is used and saved back to disk when possible.

## BroadCastChannel

`BroadCastChannel<T>` wraps Tokio broadcast channels.

It is used by the Twitch client to publish `TwitchChatMessage` values to the command worker.

## MSGQueue

`MSGQueue<T>` is a simple async FIFO queue built with `VecDeque`, `RwLock`, and `Notify`.

It is used for:

- text-to-speech messages
- synthesized audio bytes
