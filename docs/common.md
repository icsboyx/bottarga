# Common Module Documentation

This module provides utility traits and structures that are commonly used across the application. Below is an overview of the key components:

## PersistentConfig Trait

The `PersistentConfig` trait is designed to handle the persistence of configuration data. It provides the following methods:

- `save`: Saves the configuration to a file in TOML format. If the file path is invalid or inaccessible, it logs warnings and proceeds with in-memory storage.
- `check_file_path`: Ensures the parent directory for the configuration file exists, creating it if necessary.
- `load`: Loads the configuration from a file. If the file is missing or invalid, it defaults to an in-memory configuration and saves it for future use.

## BroadCastChannel Struct

The `BroadCastChannel` struct is a wrapper around Tokio's broadcast channel, enabling message broadcasting to multiple subscribers. Key methods include:

- `new`: Creates a new broadcast channel with a specified capacity.
- `init`: Initializes the channel (returns a reference to itself).
- `send_broadcast`: Sends a message to all subscribers if there are active receivers.
- `subscribe_broadcast`: Returns a new receiver for subscribing to broadcast messages.

## MSGQueue Struct

The `MSGQueue` struct is a thread-safe, asynchronous message queue implemented using a `VecDeque` and a `Notify` mechanism. Key methods include:

- `new`: Creates a new message queue.
- `push_back`: Adds a message to the back of the queue and notifies waiting consumers.
- `next`: Retrieves and removes the next message from the queue, waiting if the queue is empty.
- `next_error`: Similar to `next`, but returns a `Result` for error handling.
- `len`: Returns the current length of the queue.

These utilities are designed to simplify common tasks like configuration management, message broadcasting, and queueing in an asynchronous environment.
