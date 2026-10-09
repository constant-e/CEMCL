//! VersionManager：启动时与 .minecraft 核对、配置项默认值/省略与 current 锚定

use app::version::{ConfigMC, VersionManager};
use mc::MCType;
use std::{
    fs::{create_dir_all, read_to_string, remove_dir_all, write},
    path::PathBuf,
};

fn write_file(path: &str, content: &str) {
    if let Some(parent) = PathBuf::from(path).parent() {
        create_dir_all(parent).unwrap();
    }
    write(path, content).unwrap();
}

fn read_json(path: &str) -> serde_json::Value {
    serde_json::from_str(&read_to_string(path).unwrap()).unwrap()
}

/// 启动时与 .minecraft 核对删除无效项；配置项留空/缺省时使用默认值，保存时省略默认值
#[test]
fn load_clean_and_save() {
    // VersionManager 使用相对路径，测试在独立临时目录中进行
    let root = std::env::temp_dir().join(format!("cemcl-version-manager-{}", std::process::id()));
    let _ = remove_dir_all(&root);
    create_dir_all(&root).unwrap();
    std::env::set_current_dir(&root).unwrap();

    // .minecraft 中已有的版本
    write_file(
        ".minecraft/versions/1.19.2/1.19.2.json",
        "{\"type\": \"release\"}",
    );
    write_file(
        ".minecraft/versions/1.20.1/1.20.1.json",
        "{\"type\": \"snapshot\"}",
    );
    write_file(
        ".minecraft/versions/a_scanned/a_scanned.json",
        "{\"type\": \"old_beta\"}",
    );

    // 1.20.1 带留空与自定义配置，z_invalid 在 .minecraft 中不存在
    write_file(
        "versions.json",
        "{\n  \"current\": 1,\n  \"versions\": {\n    \"1.19.2\": {},\n    \"1.20.1\": { \"game_type\": \"release\", \"xmx\": \"4G\", \"height\": 0, \"width\": null, \"wrapper\": \"\", \"separated\": true },\n    \"z_invalid\": {}\n  }\n}",
    );

    let config = ConfigMC {
        height: 600,
        path: ".minecraft".to_string(),
        width: 800,
        wrapper: "wrap".to_string(),
        xms: "1G".to_string(),
        xmx: "2G".to_string(),
    };

    let mut manager = VersionManager::new(config).unwrap();

    // 无效项删除、.minecraft 中的新版本加入、列表按名字排序；current 仍指向 1.20.1
    let names: Vec<&str> = manager
        .get_version_list()
        .iter()
        .map(|v| v.version.as_str())
        .collect();
    assert_eq!(names, vec!["1.19.2", "1.20.1", "a_scanned"]);
    assert_eq!(manager.get_current_index(), 1);
    assert_eq!(manager.get(1).version, "1.20.1");

    // 留空/缺省的配置项使用 config.json 的默认值
    let version = manager.get(0);
    assert_eq!((version.height, version.width), (600, 800));
    assert_eq!((version.xms.as_str(), version.xmx.as_str()), ("1G", "2G"));
    assert_eq!(version.wrapper, "wrap");
    assert!(!version.separated);
    assert!(matches!(version.game_type, MCType::Release));

    let version = manager.get(1);
    assert_eq!((version.height, version.width), (600, 800));
    assert_eq!(version.xms, "1G");
    assert_eq!(version.xmx, "4G");
    assert_eq!(version.wrapper, "wrap");
    assert!(version.separated);
    assert!(matches!(version.game_type, MCType::Release));

    // 扫描出的版本也使用默认配置，类型来自版本 json
    let version = manager.get(2);
    assert_eq!((version.xms.as_str(), version.xmx.as_str()), ("1G", "2G"));
    assert_eq!(version.wrapper, "wrap");
    assert!(matches!(version.game_type, MCType::OldBeta));

    // 写回的 versions.json：无效项被删除，与默认值相同的配置项缺省
    let json = read_json("versions.json");
    let versions = json["versions"].as_object().unwrap();
    assert_eq!(versions.len(), 3);
    assert!(!versions.contains_key("z_invalid"));
    let node = versions["1.19.2"].as_object().unwrap();
    assert!(!node.contains_key("height"));
    assert!(!node.contains_key("width"));
    assert!(!node.contains_key("xms"));
    assert!(!node.contains_key("xmx"));
    assert!(!node.contains_key("wrapper"));
    let node = versions["1.20.1"].as_object().unwrap();
    assert_eq!(node["xmx"], "4G");
    assert_eq!(node["separated"], true);
    assert!(!node.contains_key("height"));
    assert!(!node.contains_key("width"));
    assert!(!node.contains_key("wrapper"));
    assert!(!node.contains_key("xms"));

    // 保存后重新加载，current 仍指向同一个版本
    manager.set_current_index(2).unwrap();
    let manager = VersionManager::new(manager.get_config().clone()).unwrap();
    assert_eq!(manager.get_current_index(), 2);
    assert_eq!(manager.get(2).version, "a_scanned");

    std::env::set_current_dir(std::env::temp_dir()).unwrap();
    let _ = remove_dir_all(&root);
}
