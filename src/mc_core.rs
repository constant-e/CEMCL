use serde_json::{Result, Value};
use std::env::consts as env;

use crate::file_tools::{exists, open_file};

// 游戏账号
pub struct Account {
    // 登录类型，直接填入
    pub account_type: String,
    // access_token,直接填入
    pub token: String,
    // uuid，直接填入
    pub uuid: String,
    // user_name，直接填入
    pub user_name: String,
}

// 游戏配置
pub struct Game {
    // 自定义参数，留空则使用默认
    pub args: String,
    // 描述，由用户输入
    pub description: String,
    // 窗口高度，-1默认
    pub height: isize,
    // java可执行文件路径
    pub java_path: String,
    // 启用版本隔离
    pub seperated: bool,
    // 游戏类型，直接填入
    pub game_type: String,
    // 游戏版本，直接填入
    pub version: String,
    // 窗口宽度，-1默认
    pub width: isize,
    // xms参数，留空默认
    pub xms: String,
    // xmx参数，留空默认
    pub xmx: String,
}

// 获取单条参数
fn add_arg(n: &Value) -> String {
    let mut result = String::new();
    for item in n.as_array().expect("[Error] mc_core: Failed to get arguments.") {
        if item.is_string() {
            // 无限制，可直接添加
            result.push_str(item.as_str().expect("[Error] mc_core: Failed to get arguments."));
            continue;
        }
        
        // 判断是否满足限制条件
        if !check_rules(item.get("rules").expect("[Error] mc_core: Failed to get rules")) {
            continue;
        }

        if item["value"].is_string() {
            // 单条参数
            result.push_str(&item["value"].as_str().expect("[Error] mc_core: Failed to get argument."));
            result.push_str(" ");
        } else {
            // 数组
            for arg in item["value"].as_array().expect("[Error] mc_core: Failed to get arguments.") {
                result.push_str(&arg.as_str().expect("[Error] mc_core: Failed to get arguments."));
                result.push_str(" ");
            }
        }
    }
    result

}

// 检查参数是否可以添加
fn check_rules(n: &Value) -> bool {
    let mut allow = true; 

    // 获取操作系统名称
    let mut os = "";
    if env::OS == "windows" || env::OS == "linux" {
        os = env::OS;
    } else if env::OS == "macOS" {
        os = "osx";
    } else {
        os = "";
        println!("[Warning] mc_core: Your OS may not be supported.");
    }

    for r in n.as_array().expect("[Error] mc_core: Failed to get arguments.") {
        if !r["features"].is_null() {
            // 暂时不支持
            allow = false;
            break;
        }
        if r["action"] == "allow" {
            if r["os"]["arch"] != env::ARCH {
                allow = false;
                break;
            }
            if r["os"]["name"] != os {
                allow = false;
                break;
            }
        } else if r["action"] == "disallow" {
            if r["os"]["arch"] == env::ARCH {
                allow = false;
                break;
            }
            if r["os"]["name"] == os {
                allow = false;
                break;
            }
        }
    }
    allow
}

// 获取MC和JVM参数
fn get_args(n: &Value) -> (String, String) {
    let mut game_args = String::new();
    let mut jvm_args = String::from("
        -XX:+UseG1GC 
        -XX:-UseAdaptiveSizePolicy 
        -XX:-OmitStackTraceInFastThrow 
        -Dfml.ignoreInvalidMinecraftCertificates=True 
        -Dfml.ignorePatchDiscrepancies=True 
        -Dlog4j2.formatMsgNoLookups=true 
    ");

    if !n.get("arguments").is_none() {
        // MC版本 >= 1.13
        game_args.push_str(&add_arg(&n["arguments"]["game"]));
        jvm_args.push_str(&add_arg(&n["arguments"]["jvm"]));
    } else {
        // MC版本 < 1.13
        game_args.push_str(n["minecraftArguments"].as_str().expect("[Error] mc_core: Couldn't get minecraftArguments."));
        game_args.push_str(" ");
        jvm_args.push_str("-Djava.library.path=${natives_directory} -cp ${classpath} ")
    }

    (game_args, jvm_args)
}

// 获取-cp参数 
fn get_classpaths(n: &Value, game_path: &String) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for item in n.as_array().expect("[Error] mc_core: Failed to get -cp arguments.") {
        let mut temp: String = game_path.clone() + "/libraries/";

        if !item["rules"].is_null() &&
            check_rules(item.get("rules").expect("[Error] mc_core: Failed to get rules")) {
            continue;
        }

        let name = String::from(item["name"].as_str().expect("msg"));
        let mut name_split: Vec<&str> = name.split(":").collect();
        temp.push_str(name_split[0].replace(".", "/").as_str());
        temp.push_str("/");
        temp.push_str(name_split[1]);
        temp.push_str("/");
        temp.push_str(name_split[2]);
        temp.push_str("/");
        temp.push_str(name_split[1]);
        temp.push_str("-");
        temp.push_str(name_split[2]);
        if name_split.len() == 4 {
            // 添加后缀
            temp.push_str("-");
            temp.push_str(name_split[3]);
        }
        temp.push_str(".jar");

        result.push(temp);
    }
    result
}

// 获取启动总命令
pub fn get_launch_command(account: &Account, game: &Game, java_path: &String, game_path: &String) -> String {
    println!("[Info] mc_core: Start.");
    let mut result = String::new();

    // 使用自定义参数
    if !game.args.is_empty() {
        result.push_str(java_path.as_str());
        result.push_str(" ");
        result.push_str(game.args.as_str());
        return result;
    }

    // 游戏目录
    let mut dir = game_path.clone();
    dir.push_str(game_path.as_str());
    dir.push_str("/versions/");
    dir.push_str(game.version.as_str());
    
    // 读取config.json
    let mut cfg_path = dir.clone();
    cfg_path.push_str("/");
    cfg_path.push_str(game.version.as_str());
    cfg_path.push_str(".json");
    let config: Value = serde_json::from_str(&open_file(&cfg_path).as_str())
        .expect(&format!("[Error] mc_core: failed to load {cfg_path}."));

    // 参数
    let mut asset_index = String::new();
    let mut classpaths: Vec<String> = Vec::new();
    let mut cp = String::new();
    let mut game_args = String::new();
    let mut jvm_args = String::new();
    let main_class = config["mainClass"].as_str().expect("[Error] mc_core: Couldn't get mainClass.");

    // 判断inheritsFrom(forge需要)
    if config.get("inheritsFrom").is_none() {
        // 无forge
        (game_args, jvm_args) = get_args(&config);
        asset_index.push_str(&config["assetIndex"]["id"].as_str().expect("[Error] mc_core: Couldn't get assetIndex."));
    } else{
        // 有forge
        let parent_version = config["inheritsFrom"].as_str().expect("[Error] mc_core: Couldn't get inheritsFrom.");
        let mut parent_path = game_path.clone();
        parent_path.push_str("/versions");
        parent_path.push_str(&parent_version);
        if exists(&parent_path) {
            let parent: Value = serde_json::from_str(&open_file(&parent_path).as_str())
                .expect(&format!("[Error] mc_core: failed to load {parent_path}."));
            (game_args, jvm_args) = get_args(&parent);
            let (temp_game_args, temp_jvm_args) = get_args(&config);
            game_args.push_str(&temp_game_args);
            jvm_args.push_str(&temp_jvm_args);
            asset_index.push_str(&parent["assetIndex"]["id"].as_str().expect("[Error] mc_core: Couldn't get assetIndex."));
            classpaths = get_classpaths(&parent["libraries"], game_path);
        } else {
            // TODO: 下载原版
        }
    }

    // 添加-cp参数
    let mut game_jar = dir.clone();
    game_jar.push_str("/");
    game_jar.push_str(game.version.as_str());
    game_jar.push_str(".jar");

    classpaths.append(&mut get_classpaths(&config["libraries"], game_path));
    classpaths.push(game_jar);
    
    let mut i = 0;
    let l = classpaths.len();
    while i < l {
        if !classpaths[i+1..l].contains(&classpaths[i]) {
            // 去重
            cp.push_str(&classpaths[i]);
            cp.push_str(":")
        }
        i += 1;
    }

    // 设置额外参数 TODO: 更多自定义
    jvm_args.push_str("${authlib_injector_param} -Xms");
    jvm_args.push_str(&game.xms);
    jvm_args.push_str(" -Xmx");
    jvm_args.push_str(&game.xmx);
    jvm_args.push_str(" ");

    game_args.push_str("--height ");
    game_args.push_str(&game.height.to_string());
    game_args.push_str(" --width ");
    game_args.push_str(&game.width.to_string());
    game_args.push_str(" ");

    // 参数添加至result
    result.push_str(&jvm_args);
    result.push_str(main_class);
    result.push_str(&game_args);

    // 替换模板
    result = result
        .replace("${assets_index_name}", &asset_index)
        .replace("${assets_root}", &(game_path.clone() + "/assets"))
        .replace("${auth_access_token}", &account.token)
        .replace("${auth_player_name}", &account.user_name)
        .replace("${auth_uuid}", &account.uuid)
        .replace("${authlib_injector_param}", "") // 暂不支持
        .replace("${classpath}", &cp)
        .replace("${classpath_separator}", ":")
        .replace("${game_pathectory}", &game_path)
        .replace("${launcher_name}", "\"CE Minecraft Launcher\"")
        .replace("${launcher_version}", "1.0.0")
        .replace("${library_directory}", &(game_path.clone() + "/libraries"))
        .replace("${natives_directory}", &(dir.clone() + "/natives"))
        .replace("${user_type}", &account.account_type)
        .replace("${version_name}", &game.version)
        .replace("${version_type}", &game.game_type);

    result
}

pub fn refresh_game_list(old_game_list: &Vec<Game>) -> Vec<Game> {
    let mut game_list: Vec<Game> = Vec::new();

    game_list
}
