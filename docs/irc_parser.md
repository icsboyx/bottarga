# IRC Parser Module Documentation

This module provides functionality to parse IRC messages, extract relevant information, and represent them as structured objects. It is designed to handle Twitch IRC messages but can be adapted for other IRC-based systems.

## Structures

### `IrcMessage`

Represents a parsed IRC message with the following fields:

- `timestamp` (`u128`): The timestamp of when the message was parsed.
- `token` (`HashMap<String, String>`): Key-value pairs extracted from the message metadata.
- `sender` (`String`): The sender of the message.
- `command` (`String`): The command or action associated with the message.
- `destination` (`String`): The target or destination of the message.
- `payload` (`String`): The main content or body of the message.

#### Methods:

- `IrcMessage::new(token, context, payload)`: Creates a new `IrcMessage` instance.

### `Context`

Represents the context of an IRC message with the following fields:

- `sender` (`String`): The sender of the message.
- `command` (`String`): The command or action associated with the message.
- `destination` (`String`): The target or destination of the message.

#### Methods:

- `Context::new(sender, command, destination)`: Creates a new `Context` instance.

## Functions

### `parse_message(msg: impl AsRef<str>) -> IrcMessage`

Parses a raw IRC message string and returns an `IrcMessage` object.

### `parse_irc_message_context(context: &str) -> Context`

Parses the context part of an IRC message and returns a `Context` object.

### `parse_irc_message_token(token: &str) -> HashMap<String, String>`

Parses the token part of an IRC message and returns a `HashMap` of key-value pairs.

## Example Usage

```rust
use std::collections::HashMap;
use crate::irc_parser::{parse_message, IrcMessage};

fn main() {
    let raw_message = "@badge-info=subscriber/6;badges=subscriber/6;color=#1E90FF;display-name=User123 :user123!user123@user123.tmi.twitch.tv PRIVMSG #channel :Hello, world!";
    let parsed_message: IrcMessage = parse_message(raw_message);

    println!("Parsed Message: {:?}", parsed_message);
}
```

## Notes

- The module assumes that the input message follows the IRC protocol format.
- It provides basic error handling by using default values for missing fields.
