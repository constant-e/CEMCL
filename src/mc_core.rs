use serde_json::{Value};
use std::str::FromStr;

mod file_tools;

// 游戏账号
struct Account {
    // 登录类型，直接填入
    account_type: String,
    // access_token,直接填入
    token: String,
    // uuid，直接填入
    uuid: String,
    // user_name，直接填入
    user_name: String,
}

struct Game {
    // 自定义参数，留空则使用默认
    args: String,
    // 描述，由用户输入
    description: String,
    // 窗口高度，-1默认
    height: i32,
    // java可执行文件路径
    java_path: String,
    // 启用版本隔离
    seperated: bool,
    // 游戏类型，直接填入
    game_type: String,
    // 游戏版本，直接填入
    version: String,
    // 窗口宽度，-1默认
    width: i32,
    // xms参数，留空默认
    xms: String,
    // xmx参数，留空默认
    xmx: String,
}

// 添加游戏
fn add_game() -> bool {
    true
}

fn get_launch_command(account: &Account, game: &Game, java_path: &String, game_dir: &String) -> String {
    println!("[Info]: [mc_core] Start.");
    let mut result: String = String::new();

    if !game.args.is_empty() {
        result.push_str(java_path.as_str());
        result.push_str(" ");
        result.push_str(game.args.as_str());
        return result;
    }

    let mut jvm_args: String = String::from("-XX:+UseG1GC 
                                             -XX:-UseAdaptiveSizePolicy 
                                             -XX:-OmitStackTraceInFastThrow 
                                             -Dfml.ignoreInvalidMinecraftCertificates=True 
                                             -Dfml.ignorePatchDiscrepancies=True 
                                             -Dlog4j2.formatMsgNoLookups=true ");
    let mut game_args: String = String::new();

    



    result
}

