//! 账号相关

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
    /// 从refresh_token刷新access_token（包含判定是否为微软登录）
    pub fn refresh(&mut self) {
        if self.account_type != "msa" { return; }
        // TODO: support online login
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

// pub fn login() -> Option<Account> {
// 
// }
