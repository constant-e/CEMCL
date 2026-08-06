//! 账号相关

#[derive(Clone, Eq, PartialEq)]
pub enum AccountType {
    Legacy,
    MSA,
    Other, // TODO: implement other account type
}

/// MC账号
#[derive(Clone)]
pub struct Account {
    /// access_token，直接填入启动参数
    pub access_token: String,

    /// 登录类型，直接填入启动参数
    pub account_type: AccountType,

    /// 用于刷新access_token
    pub refresh_token: String,

    /// uuid，直接填入启动参数
    pub uuid: String,

    /// user_name，直接填入启动参数
    pub user_name: String,
}

impl Default for Account {
    /// 创建一个默认离线账号
    fn default() -> Self {
        Account {
            access_token: String::new(),
            account_type: AccountType::Legacy,
            refresh_token: String::new(),
            uuid: String::from(uuid::Uuid::new_v4()),
            user_name: String::from("Steve"),
        }
    }
}

impl From<AccountType> for String {
    fn from(value: AccountType) -> Self {
        match value {
            AccountType::Legacy => "Legacy".to_string(),
            AccountType::MSA => "msa".to_string(),
            AccountType::Other => String::new(),
        }
    }
}
