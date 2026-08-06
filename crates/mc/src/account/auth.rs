use log::debug;
use reqwest::Client;
use serde_json::{Value, json};
use std::time::Duration;

use super::account::{Account, AccountType};

const CLIENT_ID: &str = "866440ab-2174-4ff6-8624-290608ac9bdb";

pub enum AuthError {
    AccessTokenNotFound,
    DeserializationError(serde_json::Error),
    DeviceCodeNotFound,
    MSAccessTokenNotFound,
    RefreshTokenNotFound,
    ReqwestError(reqwest::Error),
    UserCodeNotFound,
    UserNameNotFound,
    UUIDNotFound,
    VerificationUriNotFound,
    XboxTokenNotFound,
    XSTSTokenNotFound,
    XSTSUserHashNotFound,
}

impl From<reqwest::Error> for AuthError {
    fn from(err: reqwest::Error) -> Self {
        AuthError::ReqwestError(err)
    }
}

impl From<serde_json::Error> for AuthError {
    fn from(err: serde_json::Error) -> Self {
        AuthError::DeserializationError(err)
    }
}

pub enum AuthPollAction {
    /// 已完成的步数（共5步）
    Continue(u8),
    Done(Account),
}

#[derive(Clone)]
enum AuthState {
    Init,
    OAuth(String),        // Refresh token
    Xbox(String),         // Xbox token
    Xsts(String, String), // XSTS token and user hash
    MC(String),           // Minecraft access token
    Done(Account),        // Account
}

struct ReservedData {
    access_token: Option<String>,
    refresh_token: Option<String>,
}

pub struct AuthSession {
    client: Client,
    client_id: String,
    data: ReservedData,
    device_code: String,
    state: AuthState,
}

impl AuthSession {
    pub fn new(client: Client, client_id: String, device_code: String) -> Self {
        Self {
            client,
            client_id,
            data: ReservedData {
                access_token: None,
                refresh_token: None,
            },
            device_code,
            state: AuthState::Init,
        }
    }

    pub fn from_ms_access_token(
        client: Client,
        client_id: String,
        ms_access_token: String,
        refresh_token: String,
    ) -> Self {
        Self {
            client,
            client_id,
            data: ReservedData {
                access_token: None,
                refresh_token: Some(refresh_token),
            },
            device_code: String::new(),
            state: AuthState::OAuth(ms_access_token),
        }
    }

    pub async fn poll(&mut self) -> Result<AuthPollAction, AuthError> {
        match self.state.clone() {
            AuthState::Init => {
                let ms_access_token = self.oauth().await?;
                self.state = AuthState::OAuth(ms_access_token);
                Ok(AuthPollAction::Continue(1))
            }
            AuthState::OAuth(ms_access_token) => {
                let xbox_token = self.xbox(&ms_access_token).await?;
                self.state = AuthState::Xbox(xbox_token);
                Ok(AuthPollAction::Continue(2))
            }
            AuthState::Xbox(xbox_token) => {
                let (xsts_token, uhs) = self.xsts(&xbox_token).await?;
                self.state = AuthState::Xsts(xsts_token, uhs);
                Ok(AuthPollAction::Continue(3))
            }
            AuthState::Xsts(xsts_token, uhs) => {
                let access_token = self.mc(&xsts_token, &uhs).await?;
                self.state = AuthState::MC(access_token);
                Ok(AuthPollAction::Continue(4))
            }
            AuthState::MC(access_token) => {
                let (user_name, uuid) = self.profile(&access_token).await?;
                let account = Account {
                    access_token,
                    account_type: AccountType::MSA,
                    refresh_token: self
                        .data
                        .refresh_token
                        .clone()
                        .ok_or(AuthError::RefreshTokenNotFound)?,
                    uuid,
                    user_name,
                };
                self.state = AuthState::Done(account.clone());
                Ok(AuthPollAction::Done(account))
            }
            AuthState::Done(account) => Ok(AuthPollAction::Done(account.clone())),
        }
    }

    async fn oauth(&mut self) -> Result<String, AuthError> {
        debug!("Start oauth");

        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("client_id", &self.client_id),
            ("device_code", &self.device_code),
        ];

        let res = self
            .client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&params)
            .send()
            .await?;

        let json = serde_json::from_str::<Value>(&res.text().await?)?;
        let ms_access_token = json["access_token"]
            .as_str()
            .ok_or(AuthError::MSAccessTokenNotFound)?
            .to_string();
        let refresh_token = json["refresh_token"]
            .as_str()
            .ok_or(AuthError::RefreshTokenNotFound)?
            .to_string();

        self.data.refresh_token = Some(refresh_token.clone());

        debug!("Finish oauth");
        Ok(ms_access_token)
    }

    async fn xbox(&self, refresh_token: &str) -> Result<String, AuthError> {
        debug!("Start xbox");

        let send_json = json!(
            {
                "Properties": {
                    "AuthMethod": "RPS",
                    "SiteName": "user.auth.xboxlive.com",
                    "RpsTicket": format!("d={refresh_token}")
                },
                "RelyingParty": "http://auth.xboxlive.com",
                "TokenType": "JWT"
            }
        );

        let res = self
            .client
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .json(&send_json)
            .send()
            .await?;

        let recv_json = serde_json::from_str::<Value>(&res.text().await?)?;
        let xbox_token = recv_json["Token"]
            .as_str()
            .ok_or(AuthError::XboxTokenNotFound)?;

        debug!("Finish xbox");
        Ok(xbox_token.to_string())
    }

    async fn xsts(&self, xbox_token: &str) -> Result<(String, String), AuthError> {
        debug!("Start xsts");

        let send_json = json!(
            {
                "Properties": {
                    "SandboxId": "RETAIL",
                    "UserTokens": [ xbox_token ]
                },
                "RelyingParty": "rp://api.minecraftservices.com/",
                "TokenType": "JWT"
            }
        );

        let res = self
            .client
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .json(&send_json)
            .send()
            .await?;

        let recv_json = serde_json::from_str::<Value>(&res.text().await?)?;
        let xsts_token = recv_json["Token"]
            .as_str()
            .ok_or(AuthError::XSTSTokenNotFound)?;
        let uhs = recv_json["DisplayClaims"]["xui"][0]["uhs"]
            .as_str()
            .ok_or(AuthError::XSTSUserHashNotFound)?;

        debug!("Finish xsts");
        Ok((xsts_token.to_string(), uhs.to_string()))
    }

    async fn mc(&mut self, xsts_token: &str, uhs: &str) -> Result<String, AuthError> {
        debug!("Start mc");

        let send_json = json!({"identityToken": format!("XBL3.0 x={uhs};{xsts_token}")});

        let res = self
            .client
            .post("https://api.minecraftservices.com/authentication/login_with_xbox")
            .json(&send_json)
            .send()
            .await?;

        let recv_json = serde_json::from_str::<Value>(&res.text().await?)?;
        let access_token = recv_json["access_token"]
            .as_str()
            .ok_or(AuthError::AccessTokenNotFound)?
            .to_string();

        self.data.access_token = Some(access_token.clone());

        debug!("Finish mc");
        Ok(access_token)
    }

    async fn profile(&self, access_token: &str) -> Result<(String, String), AuthError> {
        debug!("Start profile");

        let res = self
            .client
            .get("https://api.minecraftservices.com/minecraft/profile")
            .bearer_auth(access_token)
            .send()
            .await?;

        let recv_json = serde_json::from_str::<Value>(&res.text().await?)?;
        let username = recv_json["name"]
            .as_str()
            .ok_or(AuthError::UserNameNotFound)?;
        let uuid = recv_json["id"].as_str().ok_or(AuthError::UUIDNotFound)?;

        debug!("Finish profile");
        Ok((username.to_string(), uuid.to_string()))
    }
}

/// Request an oauth login, return (verification_uri, user_code, AuthSession)
pub async fn request_oauth() -> Result<(String, String, AuthSession), AuthError> {
    let client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    let params = [
        ("client_id", CLIENT_ID),
        ("scope", "XboxLive.signin offline_access"),
    ];

    let res = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
        .form(&params)
        .send()
        .await?;

    let json = serde_json::from_str::<Value>(&res.text().await?)?;

    let device_code = json["device_code"]
        .as_str()
        .ok_or(AuthError::DeviceCodeNotFound)?
        .to_string();

    let verification_uri = json["verification_uri"]
        .as_str()
        .ok_or(AuthError::VerificationUriNotFound)?
        .to_string();

    let user_code = json["user_code"]
        .as_str()
        .ok_or(AuthError::UserCodeNotFound)?
        .to_string();

    Ok((
        verification_uri,
        user_code,
        AuthSession::new(client, CLIENT_ID.to_string(), device_code),
    ))
}

pub async fn request_refresh_account(refresh_token: &str) -> Result<AuthSession, AuthError> {
    let client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    // Get oauth
    let params = [
        ("grant_type", "refresh_token"),
        ("client_id", CLIENT_ID),
        ("refresh_token", refresh_token),
    ];

    let res = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .form(&params)
        .send()
        .await?;

    let json = serde_json::from_str::<Value>(&res.text().await?)?;
    let ms_access_token = json["access_token"]
        .as_str()
        .ok_or(AuthError::MSAccessTokenNotFound)?;
    let refresh_token = json["refresh_token"]
        .as_str()
        .ok_or(AuthError::RefreshTokenNotFound)?;

    Ok(AuthSession::from_ms_access_token(
        client,
        CLIENT_ID.to_string(),
        ms_access_token.to_string(),
        refresh_token.to_string(),
    ))
}
