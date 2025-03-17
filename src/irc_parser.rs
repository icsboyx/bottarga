// Parse the twitch message and return the  message object
// #![allow(dead_code, unused_variables)]
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct IrcMessage {
    pub timestamp: u128,
    pub token: HashMap<String, String>,
    pub sender: String,
    pub command: String,
    pub destination: String,
    pub payload: String,
}

impl IrcMessage {
    pub fn new(token: HashMap<String, String>, context: Context, payload: impl Into<String>) -> Self {
        Self {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
            token,
            sender: context.sender,
            command: context.command,
            destination: context.destination,
            payload: payload.into(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub sender: String,
    pub command: String,
    pub destination: String,
}

impl Context {
    pub fn new(sender: impl Into<String>, command: impl Into<String>, destination: impl Into<String>) -> Self {
        Self {
            sender: sender.into(),
            command: command.into(),
            destination: destination.into(),
        }
    }
}

pub fn parse_message(msg: impl AsRef<str>) -> IrcMessage {
    // println!("Parsing: {}", msg);
    let token;
    let context;
    let payload;

    let msg_arr;
    let msg = msg.as_ref();
    match msg {
        m if m.starts_with("@") => {
            msg_arr = msg.splitn(3, " :").collect::<Vec<&str>>();
            token = msg_arr[0].trim_start_matches('@').trim();
            context = msg_arr.get(1).unwrap_or(&"").trim();
            payload = msg_arr.get(2).unwrap_or(&"").trim();
        }
        m if m.starts_with(':') => {
            token = "";
            msg_arr = msg.splitn(3, ':').collect::<Vec<&str>>();
            context = msg_arr.get(1).unwrap_or(&"").trim();
            payload = msg_arr.get(2).unwrap_or(&"").trim();
        }

        _ => {
            token = "";
            msg_arr = msg.splitn(2, ':').collect::<Vec<&str>>();
            context = msg_arr.get(0).unwrap_or(&"").trim();
            payload = msg_arr.get(1).unwrap_or(&"").trim();
        }
    }

    let irc_message = IrcMessage::new(
        parse_irc_message_token(token),
        parse_irc_message_context(context),
        payload,
    );

    irc_message
}

fn parse_irc_message_context(context: &str) -> Context {
    let context_arr = context.split_whitespace().collect::<Vec<&str>>();
    if context_arr.len() == 1 {
        return Context {
            command: context_arr[0].to_string(),
            ..Context::default()
        };
    }
    let sender_full = context_arr.get(0).unwrap_or(&"").trim();
    let command = context_arr[1..context_arr.len() - 1].join(" ");
    let destination = context_arr.last().unwrap_or(&"").trim();

    let sender = if sender_full.contains('!') {
        sender_full
            .split('!')
            .collect::<Vec<&str>>()
            .get(0)
            .unwrap_or(&"")
            .to_string()
    } else {
        sender_full.to_string()
    };

    Context::new(sender, command, destination)
}

fn parse_irc_message_token(token: &str) -> HashMap<String, String> {
    token
        .split(';')
        .map(|item| {
            let mut key_val = item.splitn(2, '=');
            let key = key_val.next().unwrap_or_default().to_string();
            let val = key_val.next().unwrap_or_default().to_string();
            (key, val)
        })
        .collect()
}
