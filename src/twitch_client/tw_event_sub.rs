use std::fmt::Debug;

use crate::twitch_client::tw_oauth_token::{TW_TOKEN, TwitchScope};
use crate::utils::helpers::IntoJsonValue;
use crate::utils::web::post_http;

use eyre::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SubscriptionTypes {
    #[serde(rename = "channel.chat.message")]
    ChannelChatMessage,
    #[serde(rename = "user.whisper.message")]
    UserWhisperMessage,
}

impl SubscriptionTypes {
    pub fn as_str(&self) -> &str {
        match self {
            SubscriptionTypes::ChannelChatMessage => "channel.chat.message",
            SubscriptionTypes::UserWhisperMessage => "user.whisper.message",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscriptionRequest<SC: SubCondition> {
    condition: SC,
    transport: Transport,
    #[serde(rename = "type")]
    subscription_request_type: SubscriptionTypes,
    version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscriptionRequestReply {
    data: Vec<SubscriptionRequestReplyData>,
    max_total_cost: i64,
    total: i64,
    total_cost: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscriptionRequestReplyData {
    condition: ChannelSubCondition,
    cost: i64,
    created_at: String,
    id: String,
    status: String,
    transport: Transport,
    #[serde(rename = "type")]
    datum_type: String,
    version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelSubCondition {
    broadcaster_user_id: String,
    user_id: String,
}
impl SubCondition for ChannelSubCondition {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WhisperSubCondition {
    user_id: String,
}
impl SubCondition for WhisperSubCondition {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transport {
    connected_at: String,

    method: String,

    session_id: String,
}

pub trait SubCondition {}

pub async fn sub_channel_chat_message(broadcaster_user_id: impl AsRef<str>, session_id: impl AsRef<str>) -> Result<()> {
    let url = "https://api.twitch.tv/helix/eventsub/subscriptions";
    // TODO: and or multiple scopes
    let needed_auth_scopes = &[TwitchScope::UserReadChat];
    let sub_type = SubscriptionTypes::ChannelChatMessage.as_str();

    log!(
        "Subscribing to channel chat messages for broadcaster {} with session id {}",
        broadcaster_user_id.as_ref(),
        session_id.as_ref()
    );
    validate_scopes(needed_auth_scopes, false).await?;

    let subscription_request = SubscriptionRequest {
        subscription_request_type: SubscriptionTypes::ChannelChatMessage,
        version: "1".to_string(),
        condition: ChannelSubCondition {
            broadcaster_user_id: broadcaster_user_id.as_ref().to_string(),
            user_id: TW_TOKEN.user_id().await,
        },
        transport: Transport {
            method: "websocket".to_string(),
            session_id: session_id.as_ref().to_string(),
            connected_at: chrono::Utc::now().to_rfc3339(),
        },
    };

    let mut headers = TW_TOKEN.auth_headers().await;
    headers.insert("Content-Type", "application/json".parse().unwrap());

    subscribe(url, sub_type, subscription_request, headers).await
}

pub async fn sub_user_whisper_message(
    _broadcaster_user_id: impl AsRef<str>,
    session_id: impl AsRef<str>,
) -> Result<()> {
    let url = "https://api.twitch.tv/helix/eventsub/subscriptions";
    // Must have oauth scope user:read:whispers or user:manage:whispers.
    let needed_auth_scopes = &[TwitchScope::UserReadWhispers, TwitchScope::UserManageWhispers];
    let sub_type = SubscriptionTypes::UserWhisperMessage.as_str();

    log!(
        "Subscribing to user whisper messages for user {} with session id {}",
        TW_TOKEN.user_id().await,
        session_id.as_ref()
    );
    validate_scopes(needed_auth_scopes, false).await?;

    let subscription_request = SubscriptionRequest {
        subscription_request_type: SubscriptionTypes::UserWhisperMessage,
        version: "1".to_string(),
        condition: WhisperSubCondition {
            user_id: TW_TOKEN.user_id().await,
        },
        transport: Transport {
            method: "websocket".to_string(),
            session_id: session_id.as_ref().to_string(),
            connected_at: chrono::Utc::now().to_rfc3339(),
        },
    };

    let mut headers = TW_TOKEN.auth_headers().await;
    headers.insert("Content-Type", "application/json".parse()?);

    subscribe(url, sub_type, subscription_request, headers).await?;

    Ok(())
}

// Finally, a helper function to handle the subscription process,
// since it will be similar for all subscription types
async fn subscribe(
    url: &str,
    sub_type: &str,
    subscription_request: SubscriptionRequest<impl SubCondition + Debug + Serialize>,
    headers: reqwest::header::HeaderMap,
) -> std::result::Result<(), eyre::Error> {
    log!("Subscribing to {} with {:?}", sub_type, subscription_request.condition);
    let sub_reply: Value = post_http(url, Some(headers), None, Some(subscription_request.json_value()?), None)
        .await?
        .body
        .json_try_into()?;

    if let Some(sub_reply) = sub_reply["data"].as_array() {
        for sub in sub_reply {
            log!(
                "Subscription for {} created with id {}, status {}",
                sub_type,
                sub["id"],
                sub["status"]
            );
        }
    }

    Ok(())
}

pub async fn validate_scopes(scopes: &[TwitchScope], condition_and: bool) -> Result<()> {
    let mut result = false;
    for scope in scopes {
        if !TW_TOKEN.scopes().await.contains(&scope.as_str().into()) {
            if condition_and {
                bail!("Missing required scope: {}", scope.as_str());
            }
        } else {
            result = true
        }
    }
    if !result {
        bail!("Missing required scope(s): {:?}", scopes);
    }
    Ok(())
}
