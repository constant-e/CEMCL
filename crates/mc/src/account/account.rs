//! 账号相关

#[derive(Clone, Debug, Eq, PartialEq)]
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
    /// 创建一个默认离线账号，名字取 uuid 对应的默认皮肤名（与游戏内一致）
    fn default() -> Self {
        let uuid = uuid::Uuid::new_v4();
        Account {
            access_token: String::new(),
            account_type: AccountType::Legacy,
            refresh_token: String::new(),
            user_name: String::from(super::skin::default_skin_name(&uuid.to_string())),
            uuid: uuid.to_string(),
        }
    }
}

impl From<AccountType> for String {
    fn from(value: AccountType) -> Self {
        match value {
            AccountType::Legacy => "Legacy".to_string(),
            AccountType::MSA => "msa".to_string(),
            AccountType::Other => "Other".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 默认离线账号的名字取自 uuid 对应的默认皮肤
    #[test]
    fn default_account_name_matches_uuid() {
        for _ in 0..16 {
            let account = Account::default();
            assert_eq!(account.account_type, AccountType::Legacy);
            assert_eq!(
                account.user_name,
                crate::account::skin::default_skin_name(&account.uuid)
            );
        }
    }
}
