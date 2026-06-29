use crate::{
    err_log,
    twitch_client::{tw_client::TW_BROADCASTER_CHANNEL, tw_event_sub, tw_oauth_token},
    utils,
};

use eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use utils::web::{get_http, post_http};

use reqwest::header::{self, HeaderMap, HeaderValue};

use tw_event_sub::validate_scopes;
use tw_oauth_token::{BotIdentity, TW_TOKEN, TwitchScope, TwitchToken};

pub async fn validate_auth_code(bearer: impl AsRef<str>) -> Result<TwitchToken> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", bearer.as_ref()))?,
    );

    headers.insert(header::USER_AGENT, HeaderValue::from_static("tw_api_client/0.1"));
    let http_reply = get_http("https://id.twitch.tv/oauth2/validate", Some(headers), None, None).await?;
    let token = http_reply.body.json_try_into::<BotIdentity>()?;
    let tw_token = TwitchToken {
        auth_code: bearer.as_ref().to_string(),
        token_identity: token,
    };
    Ok(tw_token)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TwitchUser {
    pub broadcaster_type: String,
    pub created_at: String,
    pub description: String,
    pub display_name: String,
    pub email: Option<String>,
    pub id: String,
    pub login: String,
    pub offline_image_url: String,
    pub profile_image_url: String,
    #[serde(rename = "type")]
    pub twitch_user_type: String,
    pub view_count: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TwitchUsers {
    data: Vec<TwitchUser>,
}

pub async fn get_user_from_login(auth_headers: HeaderMap, login: impl AsRef<str>) -> Result<TwitchUser> {
    let query = [("login", login.as_ref())];

    let data: TwitchUsers = get_http(
        "https://api.twitch.tv/helix/users",
        Some(auth_headers),
        Some(&query),
        None,
    )
    .await?
    .body
    .json_try_into()?;
    data.data
        .into_iter()
        .next()
        .ok_or_else(|| eyre!("No user found with login {}", login.as_ref()))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendChatMessage {
    broadcaster_id: String,
    sender_id: String,
    message: String,
    reply_parent_message_id: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendChatMessageReplay {
    pub data: Vec<SendChatMessageReplayData>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendChatMessageReplayData {
    pub drop_reason: Option<String>,
    pub is_sent: bool,
    pub message_id: String,
}

pub async fn send_chat_message(message: impl AsRef<str>, reply_parent_message_id: Option<&str>) -> Result<()> {
    let chunks = message
        .as_ref()
        .chars()
        .collect::<Vec<char>>()
        .chunks(500)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>();

    let url = "https://api.twitch.tv/helix/chat/messages";
    let needed_auth_scopes = &[TwitchScope::UserWriteChat];
    validate_scopes(needed_auth_scopes, false).await?;

    // split message into 500 char chunks and send them sequentially, if message is longer than 500 chars
    for chunk in chunks {
        let body = SendChatMessage {
            broadcaster_id: TW_BROADCASTER_CHANNEL.get_clone().await.id,
            sender_id: TW_TOKEN.user_id().await,
            message: chunk,
            reply_parent_message_id: reply_parent_message_id.map(|s| s.to_string()),
        };
        let ret_val = post_http(
            url,
            Some(TW_TOKEN.auth_headers().await),
            None,
            Some(serde_json::to_value(body)?),
            None,
        )
        .await?
        .body
        .json_try_into::<SendChatMessageReplay>()?;

        ret_val.data.into_iter().for_each(|reply| {
            if reply.is_sent {
                log!("Message sent successfully with id {}", reply.message_id);
            } else {
                err_log!(
                    "Failed to send message, message id {}, drop reason: {}",
                    reply.message_id,
                    reply.drop_reason.unwrap_or_else(|| "None".to_string())
                );
            }
        });
    }

    Ok(())
}
