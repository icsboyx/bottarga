// Using Implicit grant flow for OAuth token
// https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#implicit-grant-flow
// Implicit grant flow
// This flow is meant for apps that don’t use a server, such as client-side JavaScript apps or mobile apps.
// To get a user access token using the implicit grant flow, navigate the user to https://id.twitch.tv/oauth2/authorize.
// For example, if your service is a website, you can add an HTML hyperlink for the user to click.
// https://id.twitch.tv/oauth2/authorize?[parameters]

use eyre::{Result, bail};

use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::{
    io::{Write, stdout},
    sync::LazyLock,
};
use tokio::sync::RwLock;

use reqwest::Url;

use crate::{CONFIG_DIR, common::PersistentConfig, twitch_client::tw_api};

pub static TW_TOKEN: LazyLock<TwitchTokenWrapper> = LazyLock::new(|| TwitchTokenWrapper::default());

static TW_CLIENT_ID: &str = "wpokl9bx114xuajjziwsoxlo787lq2";
static TW_FORCE_VERIFY: bool = false;
static TW_REDIRECT_URI: &str = "http://localhost:65432/";
static TW_RESPONSE_TYPE: &str = "token";
static DEFAULT_TW_SCOPES: &[TwitchScope] = &[
    TwitchScope::ChannelModerate,
    TwitchScope::ChannelBot,
    TwitchScope::UserBot,
    TwitchScope::UserEditBroadcast,
    TwitchScope::UserReadEmail,
    TwitchScope::UserReadEmotes,
    TwitchScope::UserReadFollows,
    TwitchScope::UserReadSubscriptions,
    TwitchScope::UserReadChat,
    TwitchScope::UserWriteChat,
    TwitchScope::UserManageWhispers,
];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TwitchScopesConfig {
    pub scopes: Vec<String>,
}

impl Default for TwitchScopesConfig {
    fn default() -> Self {
        Self {
            scopes: DEFAULT_TW_SCOPES
                .iter()
                .map(|scope| scope.as_str().to_string())
                .collect(),
        }
    }
}

impl PersistentConfig for TwitchScopesConfig {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TwitchScope {
    AnalyticsReadExtensions,
    AnalyticsReadGames,
    BitsRead,
    ChannelBot,
    ChannelManageAds,
    ChannelReadAds,
    ChannelManageBroadcast,
    ChannelReadCharity,
    ChannelManageClips,
    ChannelEditCommercial,
    ChannelReadEditors,
    ChannelManageExtensions,
    ChannelReadGoals,
    ChannelReadGuestStar,
    ChannelManageGuestStar,
    ChannelReadHypeTrain,
    ChannelManageModerators,
    ChannelReadPolls,
    ChannelManagePolls,
    ChannelReadPredictions,
    ChannelManagePredictions,
    ChannelManageRaids,
    ChannelReadRedemptions,
    ChannelManageRedemptions,
    ChannelManageSchedule,
    ChannelReadStreamKey,
    ChannelReadSubscriptions,
    ChannelManageVideos,
    ChannelReadVips,
    ChannelManageVips,
    ChannelModerate,
    ClipsEdit,
    EditorManageClips,
    ModerationRead,
    ModeratorManageAnnouncements,
    ModeratorManageAutomod,
    ModeratorReadAutomodSettings,
    ModeratorManageAutomodSettings,
    ModeratorReadBannedUsers,
    ModeratorManageBannedUsers,
    ModeratorReadBlockedTerms,
    ModeratorReadChatMessages,
    ModeratorManageBlockedTerms,
    ModeratorManageChatMessages,
    ModeratorReadChatSettings,
    ModeratorManageChatSettings,
    ModeratorReadChatters,
    ModeratorReadFollowers,
    ModeratorReadGuestStar,
    ModeratorManageGuestStar,
    ModeratorReadModerators,
    ModeratorReadShieldMode,
    ModeratorManageShieldMode,
    ModeratorReadShoutouts,
    ModeratorManageShoutouts,
    ModeratorReadSuspiciousUsers,
    ModeratorManageSuspiciousUsers,
    ModeratorReadUnbanRequests,
    ModeratorManageUnbanRequests,
    ModeratorReadVips,
    ModeratorReadWarnings,
    ModeratorManageWarnings,
    UserBot,
    UserEdit,
    UserEditBroadcast,
    UserReadBlockedUsers,
    UserManageBlockedUsers,
    UserReadBroadcast,
    UserReadChat,
    UserManageChatColor,
    UserReadEmail,
    UserReadEmotes,
    UserReadFollows,
    UserReadModeratedChannels,
    UserReadSubscriptions,
    UserReadWhispers,
    UserManageWhispers,
    UserWriteChat,
}

impl TwitchScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AnalyticsReadExtensions => "analytics:read:extensions",
            Self::AnalyticsReadGames => "analytics:read:games",
            Self::BitsRead => "bits:read",
            Self::ChannelBot => "channel:bot",
            Self::ChannelManageAds => "channel:manage:ads",
            Self::ChannelReadAds => "channel:read:ads",
            Self::ChannelManageBroadcast => "channel:manage:broadcast",
            Self::ChannelReadCharity => "channel:read:charity",
            Self::ChannelManageClips => "channel:manage:clips",
            Self::ChannelEditCommercial => "channel:edit:commercial",
            Self::ChannelReadEditors => "channel:read:editors",
            Self::ChannelManageExtensions => "channel:manage:extensions",
            Self::ChannelReadGoals => "channel:read:goals",
            Self::ChannelReadGuestStar => "channel:read:guest_star",
            Self::ChannelManageGuestStar => "channel:manage:guest_star",
            Self::ChannelReadHypeTrain => "channel:read:hype_train",
            Self::ChannelManageModerators => "channel:manage:moderators",
            Self::ChannelReadPolls => "channel:read:polls",
            Self::ChannelManagePolls => "channel:manage:polls",
            Self::ChannelReadPredictions => "channel:read:predictions",
            Self::ChannelManagePredictions => "channel:manage:predictions",
            Self::ChannelManageRaids => "channel:manage:raids",
            Self::ChannelReadRedemptions => "channel:read:redemptions",
            Self::ChannelManageRedemptions => "channel:manage:redemptions",
            Self::ChannelManageSchedule => "channel:manage:schedule",
            Self::ChannelReadStreamKey => "channel:read:stream_key",
            Self::ChannelReadSubscriptions => "channel:read:subscriptions",
            Self::ChannelManageVideos => "channel:manage:videos",
            Self::ChannelReadVips => "channel:read:vips",
            Self::ChannelManageVips => "channel:manage:vips",
            Self::ChannelModerate => "channel:moderate",
            Self::ClipsEdit => "clips:edit",
            Self::EditorManageClips => "editor:manage:clips",
            Self::ModerationRead => "moderation:read",
            Self::ModeratorManageAnnouncements => "moderator:manage:announcements",
            Self::ModeratorManageAutomod => "moderator:manage:automod",
            Self::ModeratorReadAutomodSettings => "moderator:read:automod_settings",
            Self::ModeratorManageAutomodSettings => "moderator:manage:automod_settings",
            Self::ModeratorReadBannedUsers => "moderator:read:banned_users",
            Self::ModeratorManageBannedUsers => "moderator:manage:banned_users",
            Self::ModeratorReadBlockedTerms => "moderator:read:blocked_terms",
            Self::ModeratorReadChatMessages => "moderator:read:chat_messages",
            Self::ModeratorManageBlockedTerms => "moderator:manage:blocked_terms",
            Self::ModeratorManageChatMessages => "moderator:manage:chat_messages",
            Self::ModeratorReadChatSettings => "moderator:read:chat_settings",
            Self::ModeratorManageChatSettings => "moderator:manage:chat_settings",
            Self::ModeratorReadChatters => "moderator:read:chatters",
            Self::ModeratorReadFollowers => "moderator:read:followers",
            Self::ModeratorReadGuestStar => "moderator:read:guest_star",
            Self::ModeratorManageGuestStar => "moderator:manage:guest_star",
            Self::ModeratorReadModerators => "moderator:read:moderators",
            Self::ModeratorReadShieldMode => "moderator:read:shield_mode",
            Self::ModeratorManageShieldMode => "moderator:manage:shield_mode",
            Self::ModeratorReadShoutouts => "moderator:read:shoutouts",
            Self::ModeratorManageShoutouts => "moderator:manage:shoutouts",
            Self::ModeratorReadSuspiciousUsers => "moderator:read:suspicious_users",
            Self::ModeratorManageSuspiciousUsers => "moderator:manage:suspicious_users",
            Self::ModeratorReadUnbanRequests => "moderator:read:unban_requests",
            Self::ModeratorManageUnbanRequests => "moderator:manage:unban_requests",
            Self::ModeratorReadVips => "moderator:read:vips",
            Self::ModeratorReadWarnings => "moderator:read:warnings",
            Self::ModeratorManageWarnings => "moderator:manage:warnings",
            Self::UserBot => "user:bot",
            Self::UserEdit => "user:edit",
            Self::UserEditBroadcast => "user:edit:broadcast",
            Self::UserReadBlockedUsers => "user:read:blocked_users",
            Self::UserManageBlockedUsers => "user:manage:blocked_users",
            Self::UserReadBroadcast => "user:read:broadcast",
            Self::UserReadChat => "user:read:chat",
            Self::UserManageChatColor => "user:manage:chat_color",
            Self::UserReadEmail => "user:read:email",
            Self::UserReadEmotes => "user:read:emotes",
            Self::UserReadFollows => "user:read:follows",
            Self::UserReadModeratedChannels => "user:read:moderated_channels",
            Self::UserReadSubscriptions => "user:read:subscriptions",
            Self::UserReadWhispers => "user:read:whispers",
            Self::UserManageWhispers => "user:manage:whispers",
            Self::UserWriteChat => "user:write:chat",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct BotIdentity {
    client_id: String,
    expires_in: i64,
    login: String,
    scopes: Vec<String>,
    user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TwitchToken {
    pub auth_code: String,
    pub token_identity: BotIdentity,
}

impl PersistentConfig for TwitchToken {}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Eq, PartialEq)]
pub struct StreamerChannel {
    pub name: String,
    pub id: String,
}
impl PersistentConfig for StreamerChannel {}

#[derive(Default, Debug)]
pub struct TwitchTokenWrapper {
    twitch_token: RwLock<TwitchToken>,
}

impl TwitchTokenWrapper {
    pub async fn set_token(&self, token: TwitchToken) {
        let mut twitch_token = self.twitch_token.write().await;
        *twitch_token = token;
    }

    pub async fn auth_code(&self) -> String {
        self.twitch_token.read().await.auth_code.clone()
    }

    pub async fn client_id(&self) -> String {
        self.twitch_token.read().await.token_identity.client_id.clone()
    }

    pub async fn scopes(&self) -> Vec<String> {
        self.twitch_token.read().await.token_identity.scopes.clone()
    }

    pub async fn login(&self) -> String {
        self.twitch_token.read().await.token_identity.login.clone()
    }

    pub async fn user_id(&self) -> String {
        self.twitch_token.read().await.token_identity.user_id.clone()
    }

    pub async fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.auth_code().await)).unwrap(),
        );
        headers.insert("Client-id", HeaderValue::from_str(&self.client_id().await).unwrap());
        headers
    }
}

pub async fn gen_auth_code_url() {
    let scopes = TwitchScopesConfig::load(CONFIG_DIR).await.scopes.join(" ");
    let url = Url::parse_with_params(
        "https://id.twitch.tv/oauth2/authorize",
        &[
            ("client_id", TW_CLIENT_ID),
            ("redirect_uri", TW_REDIRECT_URI),
            ("response_type", TW_RESPONSE_TYPE),
            ("scope", &scopes),
            ("force_verify", &TW_FORCE_VERIFY.to_string()),
            ("state", &chrono::Utc::now().timestamp().to_string()),
        ],
    )
    .expect("static Twitch OAuth URL is valid");
    log!("Navigate to the following URL to authorize the application:");
    log!("{}", url);
}

// Bootstrap function to initialize the Twitch token and set it in the global wrapper
pub async fn boot_token_config() -> Result<()> {
    let token = match token_init().await {
        Ok(token) => token,
        Err(e) => {
            bail!("Failed to initialize Twitch token: {}", e);
        }
    };

    TW_TOKEN.set_token(token).await;

    Ok(())
}

pub async fn get_auth_code() -> Result<String> {
    let mut user_input = String::new();
    print!("Paste the URL you were redirected to after authorization: ");
    stdout().flush()?;
    std::io::stdin().read_line(&mut user_input)?;
    Ok(user_input.trim().to_string())
}

pub async fn token_initialize() -> Result<TwitchToken> {
    log!("Initializing Twitch token...");
    gen_auth_code_url().await;
    let auth_code = get_auth_code().await?;
    log!("Auth Code received");
    let twitch_token = tw_api::validate_auth_code(auth_code).await?;
    log!("Token validated successfully");
    Ok(twitch_token)
}

async fn validate_configured_scopes(token: &TwitchToken) -> Result<()> {
    let configured_scopes = TwitchScopesConfig::load(CONFIG_DIR).await.scopes;
    let missing_scopes = configured_scopes
        .iter()
        .filter(|scope| !token.token_identity.scopes.contains(scope))
        .collect::<Vec<_>>();

    if !missing_scopes.is_empty() {
        bail!("Token is missing configured Twitch scopes: {:?}", missing_scopes);
    }

    Ok(())
}

pub async fn token_init() -> Result<TwitchToken> {
    // Load token form file.
    let mut tw_token = TwitchToken::load(CONFIG_DIR).await;

    match tw_api::validate_auth_code(tw_token.auth_code).await {
        Ok(token_identity) => match validate_configured_scopes(&token_identity).await {
            Ok(()) => {
                log!("Token loaded and validated.",);
                return Ok(token_identity);
            }
            Err(e) => {
                log!("Stored token needs new authorization: {}", e);
            }
        },
        Err(_) => log!("No valid token found, requesting new token..."),
    }
    log!("Requesting new token...");
    tw_token = token_initialize().await?;
    validate_configured_scopes(&tw_token).await?;
    tw_token.save(CONFIG_DIR).await;
    Ok(tw_token)
}
