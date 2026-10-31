//! JavaManager：启动校验、软链接去重、检测结果合并与索引映射
//!
//! 测试只使用临时目录中的假 Java（release 文件）和显式传入的检测结果，
//! 不读取本机环境变量，也不执行本机 Java。

use app::java::JavaManager;
#[cfg(unix)]
use java::java::JavaInstallation;
use std::{
    fs::{self, create_dir_all, remove_dir_all},
    sync::Mutex,
};
#[cfg(unix)]
use std::{
    fs::{read_to_string, write},
    path::{Path, PathBuf},
};

/// 测试会切换进程工作目录，同一测试二进制中的用例串行执行
static CWD_LOCK: Mutex<()> = Mutex::new(());

#[cfg(unix)]
fn write_jdk(root: &Path, name: &str, version: &str) -> PathBuf {
    let dir = root.join(name);
    create_dir_all(&dir).unwrap();
    write(dir.join("release"), format!("JAVA_VERSION=\"{version}\"\n")).unwrap();
    fs::canonicalize(&dir).unwrap()
}

#[cfg(unix)]
fn make_alias(target: &Path, link: &Path) -> PathBuf {
    std::os::unix::fs::symlink(target, link).unwrap();
    link.to_path_buf()
}

#[cfg(unix)]
#[test]
fn startup_validates_dedupes_merges_and_remaps() {
    let _cwd = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let root = std::env::temp_dir().join(format!("cemcl-java-manager-{}", std::process::id()));
    let _ = remove_dir_all(&root);
    create_dir_all(&root).unwrap();
    let old_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let jdk8 = write_jdk(&root, "jdk8", "1.8.0_402");
    let jdk17 = write_jdk(&root, "jdk17", "17.0.11");
    let jdk24 = write_jdk(&root, "jdk24", "24.0.1");
    let invalid = root.join("removed-jdk");

    // 软链接别名：只能以实际安装路径添加一次
    let jdk17_alias = make_alias(&jdk17, &root.join("jdk17-link"));
    let jdk24_alias = make_alias(&jdk24, &root.join("jdk24-link"));

    write(
        "java.json",
        serde_json::json!({
            "default": 2,
            "java_list": [
                invalid.to_string_lossy(),
                jdk8.to_string_lossy(),
                jdk17_alias.to_string_lossy(),
                jdk17.to_string_lossy(),
            ],
        })
        .to_string(),
    )
    .unwrap();

    // 检测结果由测试显式提供（真实路径 + 同一个 Java 的软链接）
    let detected = vec![
        JavaInstallation::new(jdk24.clone()).unwrap(),
        JavaInstallation::new(jdk24_alias.clone()).unwrap(),
    ];
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut manager = JavaManager::new_with_detected(tx, detected).unwrap();

    let paths: Vec<PathBuf> = manager
        .get_java_list()
        .iter()
        .map(|java| java.get_path().to_path_buf())
        .collect();
    assert!(!paths.contains(&invalid));
    assert!(!paths.contains(&jdk17_alias));
    assert!(!paths.contains(&jdk24_alias));
    assert_eq!(paths.iter().filter(|path| *path == &jdk8).count(), 1);
    assert_eq!(paths.iter().filter(|path| *path == &jdk17).count(), 1);
    assert_eq!(paths.iter().filter(|path| *path == &jdk24).count(), 1);

    // 按版本降序排列
    for pair in manager.get_java_list().windows(2) {
        assert!(pair[0].get_version() >= pair[1].get_version());
    }

    // 默认项（软链接）仍指向实际安装位置
    let default = manager.get_default().unwrap();
    assert_eq!(
        manager.get_java_list()[default as usize].get_path(),
        jdk17.as_path()
    );

    // 旧索引 -> 新索引：失效项为 None，软链接与真实路径都映射到同一项
    let index_map = manager.take_index_map();
    assert_eq!(index_map.len(), 4);
    assert_eq!(index_map[0], None);
    let new_index = |path: &Path| paths.iter().position(|p| p == path).unwrap() as u32;
    assert_eq!(index_map[1], Some(new_index(&jdk8)));
    assert_eq!(index_map[2], Some(new_index(&jdk17)));
    assert_eq!(index_map[3], Some(new_index(&jdk17)));

    // 写回文件：失效项与软链接别名已删除
    let saved: serde_json::Value =
        serde_json::from_str(&read_to_string("java.json").unwrap()).unwrap();
    let saved_paths: Vec<&str> = saved["java_list"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| path.as_str().unwrap())
        .collect();
    let invalid_str = invalid.to_string_lossy();
    let jdk17_alias_str = jdk17_alias.to_string_lossy();
    let jdk24_alias_str = jdk24_alias.to_string_lossy();
    let jdk8_str = jdk8.to_string_lossy();
    let jdk17_str = jdk17.to_string_lossy();
    let jdk24_str = jdk24.to_string_lossy();
    assert!(!saved_paths.contains(&invalid_str.as_ref()));
    assert!(!saved_paths.contains(&jdk17_alias_str.as_ref()));
    assert!(!saved_paths.contains(&jdk24_alias_str.as_ref()));
    assert_eq!(
        saved_paths
            .iter()
            .filter(|path| **path == jdk17_str.as_ref())
            .count(),
        1
    );
    assert_eq!(
        saved_paths
            .iter()
            .filter(|path| **path == jdk24_str.as_ref())
            .count(),
        1
    );
    assert!(saved_paths.contains(&jdk8_str.as_ref()));
    assert_eq!(saved["default"].as_u64(), Some(default as u64));

    // 运行时添加软链接别名时视为已存在，不产生重复项
    let count = manager.get_java_list().len();
    manager.add(jdk17_alias.clone()).unwrap();
    assert_eq!(manager.get_java_list().len(), count);

    // 运行时删除 Java：default 仍指向同一路径，旧索引正确映射
    let jdk8_index = new_index(&jdk8);
    let default_before = manager.get_default().unwrap();
    manager.del(jdk8_index).unwrap();
    let default_after = manager.get_default().unwrap();
    assert_eq!(
        manager.get_java_list()[default_after as usize].get_path(),
        jdk17.as_path()
    );
    let index_map = manager.take_index_map();
    assert_eq!(index_map[jdk8_index as usize], None);
    assert_eq!(index_map[default_before as usize], Some(default_after));

    // 运行时添加 Java：重排后 default 仍指向同一路径
    let jdk11 = write_jdk(&root, "jdk11", "11.0.22");
    let default_before = manager.get_default().unwrap();
    manager.add(jdk11.clone()).unwrap();
    let default_after = manager.get_default().unwrap();
    assert_eq!(
        manager.get_java_list()[default_after as usize].get_path(),
        jdk17.as_path()
    );
    let index_map = manager.take_index_map();
    assert_eq!(index_map[default_before as usize], Some(default_after));
    assert!(
        manager
            .get_java_list()
            .iter()
            .any(|java| java.get_path() == jdk11.as_path())
    );

    std::env::set_current_dir(old_cwd).unwrap();
    let _ = remove_dir_all(&root);
}

#[test]
fn no_java_json_starts_empty_without_detection() {
    let _cwd = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let root =
        std::env::temp_dir().join(format!("cemcl-java-manager-empty-{}", std::process::id()));
    let _ = remove_dir_all(&root);
    create_dir_all(&root).unwrap();
    let old_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut manager = JavaManager::new_with_detected(tx, Vec::new()).unwrap();
    assert!(manager.get_java_list().is_empty());
    assert_eq!(manager.get_default(), None);
    assert!(manager.take_index_map().is_empty());

    std::env::set_current_dir(old_cwd).unwrap();
    let _ = remove_dir_all(&root);
}
