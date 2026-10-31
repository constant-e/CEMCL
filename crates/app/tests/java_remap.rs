//! VersionManager：Java 列表变更后按旧索引到新索引的映射修正各版本的 java_index

use app::version::{ConfigMC, VersionManager};
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

#[test]
fn remap_java_indices_after_list_change() {
    let root = std::env::temp_dir().join(format!("cemcl-java-remap-{}", std::process::id()));
    let _ = remove_dir_all(&root);
    create_dir_all(&root).unwrap();
    std::env::set_current_dir(&root).unwrap();

    for version in ["a", "b", "c", "d", "e"] {
        write_file(
            &format!(".minecraft/versions/{version}/{version}.json"),
            "{\"type\": \"release\"}",
        );
    }

    write_file(
        "versions.json",
        "{\n  \"current\": 0,\n  \"versions\": {\n    \"a\": { \"java_index\": 0 },\n    \"b\": { \"java_index\": 1 },\n    \"c\": { \"java_index\": 2 },\n    \"d\": {},\n    \"e\": { \"java_index\": 9 }\n  }\n}",
    );

    let config = ConfigMC {
        height: 600,
        path: PathBuf::from(".minecraft"),
        width: 800,
        wrapper: String::new(),
        xms: "1G".to_string(),
        xmx: "2G".to_string(),
    };

    let mut manager = VersionManager::new(config).unwrap();

    // 旧列表 [removed, kept, moved] -> 新列表 [moved, kept]：
    // 0 引用的 Java 已删除，1 -> 2 后重排为 1，2 重排为 0，越界索引置空
    manager
        .remap_java_indices(&[None, Some(2), Some(0)])
        .unwrap();

    let versions = manager.get_version_list();
    let index = |name: &str| {
        versions
            .iter()
            .find(|v| v.version == name)
            .unwrap()
            .java_index
    };
    assert_eq!(index("a"), None);
    assert_eq!(index("b"), Some(2));
    assert_eq!(index("c"), Some(0));
    assert_eq!(index("d"), None);
    assert_eq!(index("e"), None);

    // 变化写回 versions.json
    let json: serde_json::Value =
        serde_json::from_str(&read_to_string("versions.json").unwrap()).unwrap();
    let versions = json["versions"].as_object().unwrap();
    assert!(
        !versions["a"]
            .as_object()
            .unwrap()
            .contains_key("java_index")
    );
    assert_eq!(versions["b"]["java_index"], 2);
    assert_eq!(versions["c"]["java_index"], 0);
    assert!(
        !versions["e"]
            .as_object()
            .unwrap()
            .contains_key("java_index")
    );
    assert!(
        !versions["d"]
            .as_object()
            .unwrap()
            .contains_key("java_index")
    );

    std::env::set_current_dir(std::env::temp_dir()).unwrap();
    let _ = remove_dir_all(&root);
}
