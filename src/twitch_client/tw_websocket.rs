use serde::Deserialize;
use serde_json::Value;

use crate::twitch_client::tw_event_sub::SubscriptionTypes;

#[derive(Debug, Deserialize, Clone)]
pub struct WebSocketMessage {
    pub metadata: Metadata,
    pub payload: Payload,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Metadata {
    pub message_id: String,
    pub message_type: MessageType,
    pub message_timestamp: String,
    #[serde(default)]
    pub subscription_type: Option<SubscriptionTypes>,
    #[serde(default)]
    pub subscription_version: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum MessageType {
    #[serde(rename = "session_welcome")]
    SessionWelcome,
    #[serde(rename = "session_keepalive")]
    SessionKeepAlive,
    #[serde(rename = "notification")]
    Notification,
    #[serde(rename = "session_reconnect")]
    SessionReconnect,
    #[serde(rename = "revocation")]
    Revocation,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Payload {
    #[serde(default)]
    pub session: Option<Session>,
    #[serde(default)]
    pub subscription: Option<Subscription>,
    #[serde(default)]
    pub event: Option<Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Session {
    pub id: String,
    pub status: SessionStatus,
    pub keepalive_timeout_seconds: Option<u64>,
    pub reconnect_url: Option<String>,
    pub connected_at: String,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum SessionStatus {
    #[serde(rename = "connected")]
    Connected,
    #[serde(rename = "reconnecting")]
    Reconnecting,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Subscription {
    pub id: String,
    pub status: SubscriptionStatus,
    #[serde(rename = "type")]
    pub subscription_type: SubscriptionTypes,
    pub version: String,
    pub cost: i32,
    pub condition: Value,
    pub transport: Transport,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Transport {
    pub method: TransportMethod,
    pub session_id: String,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum TransportMethod {
    #[serde(rename = "websocket")]
    Websocket,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum SubscriptionStatus {
    #[serde(rename = "enabled")]
    Enabled,
    #[serde(rename = "authorization_revoked")]
    AuthorizationRevoked,
    #[serde(rename = "user_removed")]
    UserRemoved,
    #[serde(rename = "version_removed")]
    VersionRemoved,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Message {
    pub fragments: Vec<Fragment>,
    pub text: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Fragment {
    pub cheermote: Option<serde_json::Value>,
    pub emote: Option<serde_json::Value>,
    pub mention: Option<serde_json::Value>,
    pub text: String,
    #[serde(rename = "type")]
    pub fragment_type: String,
}
