use serde_json::{Result, Value};
use std::env::consts as env;

use crate::{file_tools::{exists, list_dir, open_file}, Config};

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
fn add_arg(n: &Value) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for item in n.as_array().expect("[Error] mc_core: Failed to get arguments.") {
        if item.is_string() {
            // 无限制，可直接添加
            result.push(item.as_str().expect("[Error] mc_core: Failed to get arguments.").into());
            continue;
        }
        
        // 判断是否满足限制条件
        if !check_rules(item.get("rules").expect("[Error] mc_core: Failed to get rules")) {
            continue;
        }

        if item["value"].is_string() {
            // 单条参数
            result.push(item["value"].as_str().expect("[Error] mc_core: Failed to get argument.").into());
        } else {
            // 数组
            for arg in item["value"].as_array().expect("[Error] mc_core: Failed to get arguments.") {
                result.push(arg.as_str().expect("[Error] mc_core: Failed to get arguments.").into());
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
fn get_args(n: &Value) -> (Vec<String>, Vec<String>) {
    let mut game_args:Vec<String> = Vec::new();
    let mut jvm_args:Vec<String> = vec!["-XX:+UseG1GC".to_string(), "-XX:-UseAdaptiveSizePolicy".to_string(), "-XX:-OmitStackTraceInFastThrow".to_string(), "-Dfml.ignoreInvalidMinecraftCertificates=True".to_string(), "-Dfml.ignorePatchDiscrepancies=True".to_string(), "-Dlog4j2.formatMsgNoLookups=true".to_string()];

    if !n.get("arguments").is_none() {
        // MC版本 >= 1.13
        game_args.append(&mut add_arg(&n["arguments"]["game"]));
        jvm_args.append(&mut add_arg(&n["arguments"]["jvm"]));
    } else {
        // MC版本 < 1.13
        let args: Vec<&str> = n["minecraftArguments"].as_str().expect("[Error] mc_core: Couldn't get minecraftArguments.").split(" ").collect();
        for arg in args {
            game_args.push(arg.into());
        }
        jvm_args.push("-Djava.library.path=${natives_directory}".into());
        jvm_args.push("-cp".into());
        jvm_args.push("${classpath}".into());
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

        let name = String::from(item["name"].as_str().expect("Error"));
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
pub fn get_launch_command(account: &Account, game: &Game, game_path: &String) -> Vec<String> {
    println!("[Info] mc_core: Start.");
    // 使用自定义参数
    // if !game.args.is_empty() {
    //     return game.args.clone();
    // }
    let mut result: Vec<String> = Vec::new();

    // 游戏目录
    let mut dir = game_path.clone();
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
    let mut game_args:Vec<String> = Vec::new();
    let mut jvm_args:Vec<String> = Vec::new();
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
            let (mut temp_game_args, mut temp_jvm_args) = get_args(&config);
            game_args.append(&mut temp_game_args);
            jvm_args.append(&mut temp_jvm_args);
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
    jvm_args.append(&mut vec![/*"${authlib_injector_param}".to_string(), */"-Xms".to_string() + &game.xms, "-Xmx".to_string() + &game.xmx]);
    game_args.append(&mut vec!["--height ".to_string(), game.height.to_string(), "--width ".into(), game.width.to_string()]);

    // 参数添加至result
    result.append(&mut jvm_args);
    result.push(main_class.to_string());
    result.append(&mut game_args);

    // 替换模板
    for item in result.iter_mut() {
        //TODO 优化替换
        *item = item
            .replace("${assets_index_name}", &asset_index)
            .replace("${assets_root}", &(game_path.clone() + "/assets"))
            .replace("${auth_access_token}", &account.token)
            .replace("${auth_player_name}", &account.user_name)
            .replace("${auth_uuid}", &account.uuid)
            // .replace("${authlib_injector_param}", "") // 暂不支持
            .replace("${classpath}", &cp)
            .replace("${classpath_separator}", ":")
            .replace("${game_directory}", &game_path)
            .replace("${launcher_name}", "\"CE Minecraft Launcher\"")
            .replace("${launcher_version}", "1.0.0")
            .replace("${library_directory}", &(game_path.clone() + "/libraries"))
            .replace("${natives_directory}", &(dir.clone() + "/natives"))
            .replace("${user_type}", &account.account_type)
            .replace("${version_name}", &game.version)
            .replace("${version_type}", &game.game_type);
    }
    result
}

pub fn refresh_game_list(old_game_list: &Vec<Game>, config: &Config) -> Vec<Game> {
    // TODO 优化index.json的逻辑，直接储存至单个版本内
    let mut game_list: Vec<Game> = Vec::new();
    let dir = config.game_path.clone() + "/versions";

    if !exists(&dir) {
        println!("{dir}: err");
        return game_list;
    }

    for path in list_dir(&dir) {
        let json: Value = serde_json::from_str(&open_file(&(dir.clone() + "/" + path.as_str() + "/" + path.as_str() + ".json")).as_str())
            .expect(&format!("[Error] mc_core: failed to load {path}."));
        if json["type"].is_null() {
            continue;
        }
        let game = Game {
            args: "".into(),
            description: "".into(),
            height: config.height,
            java_path: config.java_path.clone(),
            seperated: false,
            game_type: json["type"].as_str().expect("").to_string(),
            version: path.into(),
            width: config.width,
            xms: config.xms.clone(),
            xmx: config.xmx.clone()
        };
        game_list.push(game);
    };
    game_list
}
