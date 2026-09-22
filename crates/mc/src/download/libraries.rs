//! Download libraries

use log::error;
use serde_json::Value;
use std::collections::HashMap;
use std::env::consts as env;
use std::fs::{File, create_dir_all, exists};
use std::io::copy;

use utils::{check_rules, get_parent_dir};

use super::{DownloadError, DownloadTask, TaskInfo};

/// 下载library
fn download_lib(save_path: &str, node: &Value, mirror: &str) -> Result<TaskInfo, DownloadError> {
    let dir = get_parent_dir(&save_path);
    if !exists(&dir)? {
        create_dir_all(&dir)?;
    }
    let mut url = node["url"]
        .as_str()
        .ok_or(DownloadError::DataInvalid)?
        .to_string();
    url = url.replace("https://libraries.minecraft.net", &mirror);
    Ok(TaskInfo {
        url,
        save_path: save_path.to_string(),
    })
}

/// 下载libraries，node: mc json["libraries"]，返回Tasks
pub fn download_libraries(
    node: &Value,
    path: &str,
    game_dir: &str,
    mirror: &str,
    fabric_mirror: &str,
) -> Result<Vec<DownloadTask>, DownloadError> {
    let mut tasks = Vec::new();
    for item in node.as_array().ok_or(DownloadError::DataInvalid)? {
        let (node, path, game_dir, mirror) = (
            item.clone(),
            path.to_string(),
            game_dir.to_string(),
            mirror.to_string(),
        );
        let lib_dir = path.to_string() + "/libraries";
        let os = if env::OS == "macOS" { "osx" } else { env::OS };
        let natives_dir = game_dir.to_string() + "/natives-" + os + "-" + env::ARCH;
        if node["rules"].is_array() {
            if !check_rules(&node["rules"]) {
                continue;
            }
        }
        // Add natives for old versions
        if node["natives"][os].is_string() && node["downloads"]["classifiers"].is_object() {
            let arch = if env::ARCH.contains("64") { "64" } else { "32" };
            let key = node["natives"][os]
                .as_str()
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invaild data",
                ))?
                .replace("${arch}", arch);
            let node = &node["downloads"]["classifiers"][&key];
            let save_path =
                lib_dir.clone() + "/" + node["path"].as_str().ok_or(DownloadError::DataInvalid)?; // 储存位置
            if !exists(&save_path)? {
                let task_info = download_lib(&save_path, node, &mirror)?;
                let natives_dir_clone = natives_dir.clone();
                tasks.push(DownloadTask {
                    url: task_info.url,
                    save_path: task_info.save_path,
                    on_finish: Some(Box::new(move || {
                        extract_lib_logged(&natives_dir_clone, &save_path);
                    })),
                });
            } else {
                // TODO: check hash
                extract_lib_logged(&natives_dir, &save_path);
            }
        }
        if node["downloads"]["artifact"].is_object() {
            let save_path = lib_dir.clone()
                + "/"
                + node["downloads"]["artifact"]["path"]
                    .as_str()
                    .ok_or(DownloadError::DataInvalid)?;
            if !exists(&save_path)? {
                let task_info = download_lib(&save_path, &node["downloads"]["artifact"], &mirror)?;
                let natives_dir_clone = natives_dir.clone();
                tasks.push(DownloadTask {
                    url: task_info.url,
                    save_path: task_info.save_path,
                    on_finish: Some(Box::new(move || {
                        extract_lib_logged(&natives_dir_clone, &save_path);
                    })),
                });
            } else {
                // TODO: check hash
                extract_lib_logged(&natives_dir, &save_path);
            }
        } else {
            if let Some(url) = node["url"].as_str() {
                if url == "https://maven.fabricmc.net/" {
                    // Fabric
                    let mut path = String::new();
                    let name = node["name"].as_str().ok_or(DownloadError::DataInvalid)?;
                    let split_1: Vec<&str> = name.split(":").collect();
                    let split_2: Vec<&str> = split_1[0].split(".").collect();
                    for name in split_2 {
                        path = path + name + "/";
                    }
                    for i in 1..split_1.len() {
                        let name = split_1[i];
                        path = path + name + "/";
                    }
                    path = path + split_1[1] + "-" + split_1[2] + ".jar";
                    let local_path = lib_dir.clone() + "/" + &path;
                    if !exists(&local_path)? {
                        let url = fabric_mirror.to_string() + "/" + &path;
                        tasks.push(DownloadTask::new(url, local_path, None));
                    } else {
                        // TODO: check hash
                    }
                }
            }
        }
    }

    Ok(tasks)
}

/// 解压出natives，失败时记录日志（不阻断启动）
fn extract_lib_logged(natives_dir: &str, local_path: &str) {
    if let Err(e) = extract_lib(natives_dir, local_path) {
        error!("Failed to extract natives from {local_path}. Reason: {e}.");
    }
}

/// 本地库的平台匹配度：`(架构匹配度, 是否匹配本机系统)`，越大越优先。
/// 返回`None`表示该文件属于其它平台。
///
/// 同名文件在整合jar中常有多个平台/架构的版本（如jna、lz4-java），
/// 需要挑出与本机匹配的那个，否则可能把其它架构的库放进natives目录。
fn native_score(name: &str) -> Option<(u8, bool)> {
    // (平台, 关键字)，长关键字在前，避免"x86"抢到"x86-64"
    const OS: &[(&str, &[&str])] = &[
        ("windows", &["win32", "windows"]),
        ("macos", &["darwin", "macos", "osx"]),
        ("linux", &["linux"]),
        ("freebsd", &["freebsd"]),
        ("openbsd", &["openbsd"]),
        ("dragonflybsd", &["dragonflybsd"]),
        ("sunos", &["sunos"]),
    ];
    // 本机架构：2 = 完全相同，1 = 同族或未标明
    let (exact, family): (&[&str], &[&str]) = match env::ARCH {
        "x86_64" => (
            &["x8664", "amd64", "x64"],
            &["x86", "i386", "i586", "i686", "x32"],
        ),
        "x86" => (
            &["x86", "i386", "i586", "i686", "x32"],
            &["x8664", "amd64", "x64"],
        ),
        "aarch64" => (&["aarch64", "arm64"], &["arm", "armel", "armv7"]),
        "arm" => (&["arm", "armel", "armv7"], &["aarch64", "arm64"]),
        "riscv64" => (&["riscv64"], &[]),
        "powerpc64" => (&["ppc64", "ppc64le", "powerpc64"], &["ppc"]),
        "s390x" => (&["s390x"], &[]),
        "loongarch64" => (&["loongarch64"], &[]),
        "mips64el" => (&["mips64el"], &["mips"]),
        _ => (&[], &[]),
    };
    // 其它架构的关键字
    const OTHER_ARCH: &[&str] = &[
        "x8664",
        "amd64",
        "x64",
        "x86",
        "i386",
        "i586",
        "i686",
        "x32",
        "aarch64",
        "arm64",
        "arm",
        "armel",
        "armv7",
        "riscv64",
        "ppc64",
        "ppc64le",
        "powerpc64",
        "ppc",
        "s390x",
        "loongarch64",
        "mips64el",
        "mips",
        "sparc",
        "sparcv9",
        "ia64",
    ];

    let compact: String = name
        .to_lowercase()
        .chars()
        .filter(|c| !matches!(c, '-' | '_' | '.'))
        .collect();

    let mut os_matched = false;
    for (platform, keys) in OS {
        if keys.iter().any(|k| compact.contains(k)) {
            if *platform == env::OS {
                os_matched = true;
            } else {
                return None;
            }
        }
    }

    let arch = if exact.iter().any(|k| compact.contains(k)) {
        2
    } else if family.iter().any(|k| compact.contains(k)) {
        1
    } else if OTHER_ARCH.iter().any(|k| compact.contains(k)) {
        return None;
    } else {
        1
    };

    Some((arch, os_matched))
}

/// 解压出natives
///
/// 只从jar中读出需要的本地库（`.dll`/`.dylib`/`.so`），目标文件已存在时跳过，
/// 同名文件有多个平台/架构版本时取与本机最匹配的一个。
/// 不再把整个jar解压到临时目录（旧实现每次启动都要在临时目录写/删上万个文件）。
pub fn extract_lib(natives_dir: &str, local_path: &str) -> Result<(), DownloadError> {
    let mut zip = zip::ZipArchive::new(File::open(local_path)?).map_err(std::io::Error::from)?;

    // 文件名 -> ((架构匹配度, 系统是否匹配), 条目下标)
    let mut selected: HashMap<String, ((u8, bool), usize)> = HashMap::new();
    for i in 0..zip.len() {
        let entry = zip.by_index(i).map_err(std::io::Error::from)?;
        if entry.is_dir() {
            continue;
        }

        let name = entry.name();
        if !(name.ends_with(".dll") || name.ends_with(".dylib") || name.ends_with(".so")) {
            continue;
        }
        let Some(score) = native_score(name) else {
            continue;
        };

        let file_name = name.rsplit('/').next().unwrap_or_default().to_string();
        let best = selected.entry(file_name).or_insert((score, i));
        if score > best.0 {
            *best = (score, i);
        }
    }
    if selected.is_empty() {
        return Ok(());
    }

    create_dir_all(natives_dir)?;
    for (file_name, (_, i)) in selected {
        let target_path = natives_dir.to_string() + "/" + &file_name;
        if exists(&target_path)? {
            continue;
        }

        let mut entry = zip.by_index(i).map_err(std::io::Error::from)?;
        let mut target = File::create(&target_path)?;
        copy(&mut entry, &mut target)?;
        // 保留jar中记录的可执行权限
        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&target_path, std::fs::Permissions::from_mode(mode & 0o777))?;
        }
    }
    Ok(())
}
