//! 账号头像（皮肤头部正面）：启动时先给出缓存或默认头像，再在后台更新正版账号的皮肤

use log::{error, warn};
use mc::account::{
    Account, AccountType,
    skin::{self, Avatar},
};
use std::{
    collections::HashMap,
    fs::{self, create_dir_all, read, remove_file},
    io::ErrorKind,
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// 头像缓存目录（相对启动器工作目录）
const CACHE_DIR: &str = "avatar";

pub struct AvatarManager {
    /// 已加载的头像，按 uuid 索引
    avatars: HashMap<String, Avatar>,
    /// 后台皮肤更新的结果（uuid、头像），由运行时的事件循环接收
    update_receiver: Option<UnboundedReceiver<(String, Avatar)>>,
    update_sender: UnboundedSender<(String, Avatar)>,
}

impl AvatarManager {
    pub fn new() -> Self {
        // 缓存目录不存在就创建，失败只影响头像缓存
        if let Err(e) = create_dir_all(CACHE_DIR) {
            error!("Failed to create {CACHE_DIR}. Reason: {e}.");
        }

        let (update_sender, update_receiver) = unbounded_channel();
        Self {
            avatars: HashMap::new(),
            update_receiver: Some(update_receiver),
            update_sender,
        }
    }

    /// 取出后台更新结果的接收端
    pub fn take_update_receiver(&mut self) -> Option<UnboundedReceiver<(String, Avatar)>> {
        self.update_receiver.take()
    }

    pub fn get(&self, account: &Account) -> Avatar {
        self.avatars
            .get(&account.uuid)
            .cloned()
            .unwrap_or_else(|| default_avatar(account))
    }

    /// 加载账号列表的头像
    pub fn load(&mut self, accounts: &[Account]) {
        for account in accounts {
            self.load_account(account);
        }
    }

    /// 加载单个账号的头像，新增、编辑账号后也要调用
    pub fn load_account(&mut self, account: &Account) {
        let avatar = if account.account_type == AccountType::MSA {
            // 正版账号先用缓存（没有缓存就先用默认头像），后台再更新皮肤
            self.refresh(&account.uuid);
            read_cache(&account.uuid).unwrap_or_else(skin::fallback_avatar)
        } else {
            default_avatar(account)
        };
        self.avatars.insert(account.uuid.clone(), avatar);
    }

    /// 后台更新正版账号的皮肤，不影响其他功能
    fn refresh(&self, uuid: &str) {
        let update_sender = self.update_sender.clone();
        let uuid = String::from(uuid);
        tokio::spawn(async move {
            let avatar = match skin::request_skin(&uuid).await {
                // 没有自定义皮肤时游戏会退回按 uuid 选的默认皮肤
                Ok(None) => skin::default_avatar(&uuid),
                Ok(Some(skin)) => match skin::avatar_from_skin(&skin) {
                    Ok(avatar) => avatar,
                    // 皮肤无效时游戏同样会退回默认皮肤
                    Err(e) => {
                        warn!("Invalid skin of {uuid}. Reason: {e}.");
                        skin::default_avatar(&uuid)
                    }
                },
                // 网络失败：保留已有的头像
                Err(e) => {
                    warn!("Failed to update skin of {uuid}. Reason: {e}.");
                    return;
                }
            };

            save_cache(&uuid, &avatar);
            if update_sender.send((uuid, avatar)).is_err() {
                error!("Failed to send avatar update.");
            }
        });
    }

    /// 后台更新完成后写入头像
    pub fn insert(&mut self, uuid: String, avatar: Avatar) {
        self.avatars.insert(uuid, avatar);
    }

    /// 删除账号时丢弃头像与缓存
    pub fn remove(&mut self, uuid: &str) {
        self.avatars.remove(uuid);

        let path = cache_path(uuid);
        if let Err(e) = remove_file(&path)
            && e.kind() != ErrorKind::NotFound
        {
            error!("Failed to remove {path}. Reason: {e}.");
        }
    }
}

/// 账号没有皮肤（或还没有拿到皮肤）时使用的头像
fn default_avatar(account: &Account) -> Avatar {
    match account.account_type {
        // 离线账号没有皮肤，头像与游戏内一致：按 uuid 选默认皮肤
        AccountType::Legacy => skin::default_avatar(&account.uuid),
        AccountType::MSA | AccountType::Other => skin::fallback_avatar(),
    }
}

/// 缓存文件路径：uuid 统一成小写、不带连字符
fn cache_path(uuid: &str) -> String {
    format!("{CACHE_DIR}/{}.png", uuid.replace('-', "").to_lowercase())
}

/// 读取缓存的头像；没有缓存或缓存损坏时返回 None
fn read_cache(uuid: &str) -> Option<Avatar> {
    let path = cache_path(uuid);
    match Avatar::from_png(&read(&path).ok()?) {
        Ok(avatar) => Some(avatar),
        Err(e) => {
            warn!("Ignore invalid avatar cache {path}. Reason: {e}.");
            None
        }
    }
}

/// 保存头像缓存，失败只记日志：缓存只影响下次启动时的显示速度
fn save_cache(uuid: &str, avatar: &Avatar) {
    let png = match avatar.to_png() {
        Ok(png) => png,
        Err(e) => {
            error!("Failed to encode avatar of {uuid}. Reason: {e}.");
            return;
        }
    };

    let path = cache_path(uuid);
    if let Err(e) = fs::write(&path, png) {
        error!("Failed to write {path}. Reason: {e}.");
    }
}
