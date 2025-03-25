# Users Module Documentation

The `users.rs` module is responsible for managing user data and configurations in the application. It provides functionality to add, update, and retrieve user information, as well as persist this data.

## Key Components

### `UsersDB`

The `UsersDB` struct is a database for storing user information. It uses a `HashMap` to map user nicknames to their corresponding `User` objects.

#### Methods

- **`init(config_dir: Option<&str>) -> UsersDB`**  
  Initializes the `UsersDB` by loading data from the specified configuration directory.

- **`warm_up(&self)`**  
  Preloads all users. This method is called when the bot starts.

- **`add_new_user(&mut self, nick: impl AsRef<str>) -> User`**  
  Adds a new user to the database with the given nickname.

- **`update_user(&mut self, nick: impl AsRef<str>, speech_config: SpeechConfig) -> User`**  
  Updates the speech configuration for an existing user or creates a new user if the nickname does not exist.

- **`get_user(&mut self, nick: impl AsRef<str>) -> User`**  
  Retrieves a user by nickname. If the user does not exist, a new user is created.

### `User`

The `User` struct represents an individual user with a nickname and speech configuration.

#### Fields

- **`nick: String`**  
  The nickname of the user.

- **`speech_config: SpeechConfig`**  
  The speech configuration associated with the user.

#### Methods

- **`new(nick: impl AsRef<str>) -> Self`**  
  Creates a new `User` with the given nickname and a default speech configuration.

- **`get_speech_config(&self) -> &SpeechConfig`**  
  Returns a reference to the user's speech configuration.

## Persistent Storage

The `UsersDB` implements the `PersistentConfig` trait, enabling it to save and load user data from a persistent storage location defined by `CONFIG_DIR`.

## Dependencies

- **`msedge_tts::tts::SpeechConfig`**  
  Used for managing speech configurations for users.

- **`tokio::sync::RwLock`**  
  Provides thread-safe read-write access to the `UsersDB`.

- **`serde::{Deserialize, Serialize}`**  
  Enables serialization and deserialization of user data.

## Example Usage

```rust
use crate::users::{UsersDB, USER_DB};

#[tokio::main]
async fn main() {
    let mut db = USER_DB.write().await;

    // Add a new user
    let user = db.add_new_user("example_user").await;
    println!("Added user: {:?}", user);

    // Update a user's speech configuration
    let updated_user = db.update_user("example_user", SpeechConfig::default()).await;
    println!("Updated user: {:?}", updated_user);

    // Retrieve a user
    let retrieved_user = db.get_user("example_user").await;
    println!("Retrieved user: {:?}", retrieved_user);
}
```

## Notes

- The `UsersDB` is thread-safe and uses `RwLock` for concurrent access.
- Default speech configurations are filtered to use Italian voices (`it-IT`).
