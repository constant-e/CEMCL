//! 账号相关

use log::debug;
use serde_json::{Value, json};
use std::time::Duration;

/// MC账号
#[derive(Clone)]
pub struct Account {
    /// access_token，直接填入启动参数
    pub access_token: String,

    /// 登录类型，直接填入启动参数
    pub account_type: String,

    /// 用于刷新access_token
    pub refresh_token: String,

    /// uuid，直接填入启动参数
    pub uuid: String,

    /// user_name，直接填入启动参数
    pub user_name: String,
}

impl Account {
    pub async fn new(device_code: &str, ui_weak: slint::Weak<crate::AppWindow>) -> Option<Self> {
        let client_id = "866440ab-2174-4ff6-8624-290608ac9bdb";
        let client = reqwest::Client::new();
        let mut account = Account::default();

        // Get oauth
        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("client_id", client_id),
            ("device_code", device_code),
        ];
        let ms_res = client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&params)
            .send()
            .await
            .ok()?;

        let ms_json = serde_json::from_str::<Value>(&ms_res.text().await.ok()?).ok()?;
        let ms_token = ms_json["access_token"].as_str()?;
        account.refresh_token = String::from(ms_json["refresh_token"].as_str()?);

        ui_weak
            .upgrade_in_event_loop(|ui| {
                ui.set_progress(0.2);
            })
            .ok()?;

        debug!("Finish oauth");

        Account::login(&mut account, ms_token, ui_weak).await?;

        Some(account)
    }

    async fn login(
        &mut self,
        ms_token: &str,
        ui_weak: slint::Weak<crate::AppWindow>,
    ) -> Option<()> {
        let client = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(5))
            .build()
            .ok()?;

        // Get xbox
        let xbox_send_json = json!(
            {
                "Properties": {
                    "AuthMethod": "RPS",
                    "SiteName": "user.auth.xboxlive.com",
                    "RpsTicket": format!("d={ms_token}")
                },
                "RelyingParty": "http://auth.xboxlive.com",
                "TokenType": "JWT"
            }
        );
        let xbox_res = client
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .json(&xbox_send_json)
            .send()
            .await
            .ok()?;
        let xbox_json = serde_json::from_str::<Value>(&xbox_res.text().await.ok()?).ok()?;
        let xbox_token = xbox_json["Token"].as_str()?;

        ui_weak
            .upgrade_in_event_loop(|ui| {
                ui.set_progress(0.4);
            })
            .ok()?;

        debug!("Finish xbox");

        // Get xsts
        let xsts_send_json = json!(
            {
                "Properties": {
                    "SandboxId": "RETAIL",
                    "UserTokens": [ xbox_token ]
                },
                "RelyingParty": "rp://api.minecraftservices.com/",
                "TokenType":"JWT"
            }
        );
        let xsts_res = client
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .json(&xsts_send_json)
            .send()
            .await
            .ok()?;
        let xsts_json = serde_json::from_str::<Value>(&xsts_res.text().await.ok()?).ok()?;
        let xsts_token = xsts_json["Token"].as_str()?;
        let uhs = xsts_json["DisplayClaims"]["xui"][0]["uhs"].as_str()?;

        ui_weak
            .upgrade_in_event_loop(|ui| {
                ui.set_progress(0.6);
            })
            .ok()?;

        debug!("Finish xsts");

        // Get MC
        let mc_send_json = json!({"identityToken": format!("XBL3.0 x={uhs};{xsts_token}")});
        let mc_res = client
            .post("https://api.minecraftservices.com/authentication/login_with_xbox")
            .json(&mc_send_json)
            .send()
            .await
            .ok()?;
        let mc_json = serde_json::from_str::<Value>(&mc_res.text().await.ok()?).ok()?;
        let access_token = mc_json["access_token"].as_str()?;

        ui_weak
            .upgrade_in_event_loop(|ui| {
                ui.set_progress(0.8);
            })
            .ok()?;

        debug!("Finish mc");

        // Get MC profile
        let header = format!("Bearer {access_token}");
        let profile_res = client
            .get("https://api.minecraftservices.com/minecraft/profile")
            .header("Authorization", header)
            .send()
            .await
            .ok()?;
        let profile_json = serde_json::from_str::<Value>(&profile_res.text().await.ok()?).ok()?;

        ui_weak
            .upgrade_in_event_loop(|ui| {
                ui.set_progress(1.0);
            })
            .ok()?;

        debug!("Finish profile");

        self.access_token = String::from(access_token);
        self.account_type = String::from("msa");
        self.uuid = String::from(profile_json["id"].as_str()?);
        self.user_name = String::from(profile_json["name"].as_str()?);

        Some(())
    }

    /// 从refresh_token刷新access_token（包含判定是否为微软登录），并在UI更新进度
    pub async fn refresh(&mut self, ui_weak: slint::Weak<crate::AppWindow>) -> Option<()> {
        if self.account_type != "msa" {
            return Some(());
        }
        let client_id = "866440ab-2174-4ff6-8624-290608ac9bdb";
        let client = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(5))
            .build()
            .ok()?;

        // Get oauth
        let params = [
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", &self.refresh_token),
        ];
        let res = client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .form(&params)
            .send()
            .await
            .ok()?;

        let json = serde_json::from_str::<Value>(&res.text().await.ok()?).ok()?;
        let ms_token = json["access_token"].as_str()?;

        ui_weak
            .upgrade_in_event_loop(|ui| {
                ui.set_progress(0.2);
            })
            .ok()?;

        debug!("Finish oauth");

        self.login(ms_token, ui_weak).await?;

        Some(())
    }
}

impl Default for Account {
    /// 创建一个默认离线账号
    fn default() -> Self {
        Account {
            access_token: String::new(),
            account_type: String::from("Legacy"),
            refresh_token: String::new(),
            uuid: String::from(uuid::Uuid::new_v4()),
            user_name: String::from("Steve"),
        }
    }
}

// request an oauth login, and return (message, device_code, user_code, link)
pub async fn init_oauth() -> Option<(String, String, String, String)> {
    let client_id = "866440ab-2174-4ff6-8624-290608ac9bdb";
    let client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(5))
        .build()
        .ok()?;
    let params = [
        ("client_id", client_id),
        ("scope", "XboxLive.signin offline_access"),
    ];
    let res = client
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
        .form(&params)
        .send()
        .await
        .ok()?;

    let json = serde_json::from_str::<Value>(&res.text().await.ok()?).ok()?;

    Some((
        String::from(json["message"].as_str()?),
        String::from(json["device_code"].as_str()?),
        String::from(json["user_code"].as_str()?),
        String::from(json["verification_uri"].as_str()?),
    ))
}
