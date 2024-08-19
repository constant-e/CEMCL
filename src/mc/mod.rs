//! mc MC相关

/// 游戏账号相关
pub mod account;
/// 游戏配置相关
pub mod game;
/// 启动参数相关
pub mod launch;

// 游戏账号
pub struct Account {
    // 登录类型，直接填入
    pub account_type: std::cell::RefCell<String>,
    // access_token,直接填入
    pub token: std::cell::RefCell<String>,
    // uuid，直接填入
    pub uuid: std::cell::RefCell<String>,
    // user_name，直接填入
    pub user_name: std::cell::RefCell<String>,
}

// 游戏配置
pub struct Game {
    // 自定义参数，留空则使用默认
    pub args: std::cell::RefCell<String>,
    // 描述，由用户输入
    pub description: std::cell::RefCell<String>,
    // 窗口高度
    pub height: std::cell::RefCell<String>,
    // java可执行文件路径
    pub java_path: std::cell::RefCell<String>,
    // 启用版本隔离
    pub seperated: std::cell::RefCell<bool>,
    // 游戏类型，直接填入
    pub game_type: std::cell::RefCell<String>,
    // 游戏版本，直接填入
    pub version: std::cell::RefCell<String>,
    // 窗口宽度
    pub width: std::cell::RefCell<String>,
    // xms参数
    pub xms: std::cell::RefCell<String>,
    // xmx参数
    pub xmx: std::cell::RefCell<String>,
}
