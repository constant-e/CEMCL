//! 文件相关

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

/// 获取文件所在文件夹
pub fn get_parent_dir(path: &String) -> String {
    let mut vec: Vec<&str> = path.split("/").collect();
    if vec.len() == 1 {
        return String::new();
    }
    vec.pop().unwrap();
    let mut dir = String::new();
    for item in vec {
        dir.push_str(item);
        dir.push('/');
    }
    dir.pop();
    dir
}

/// 列出目录下所有文件和文件夹
pub fn list_all(path: &String) -> std::io::Result<Vec<String>> {
    let mut result: Vec<String> = Vec::new();
    for entry in fs::read_dir(&Path::new(path))? {
        let entry = entry?;
        let path = entry.path();
        result.push(
            path.file_name()
                .ok_or(ErrorKind::InvalidData)?
                .to_str()
                .ok_or(ErrorKind::InvalidData)?
                .into(),
        );
    }
    Ok(result)
}

/// 列出目录下所有文件夹
pub fn list_dir(path: &String) -> std::io::Result<Vec<String>> {
    let mut result: Vec<String> = Vec::new();
    for entry in fs::read_dir(&Path::new(path))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        result.push(
            path.file_name()
                .ok_or(ErrorKind::InvalidData)?
                .to_str()
                .ok_or(ErrorKind::InvalidData)?
                .into(),
        );
    }
    Ok(result)
}

/// 递归列出目录下所有文件
pub fn list_file(path: &String) -> std::io::Result<Vec<String>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(&Path::new(path))? {
        let entry = entry?;
        let entry_path = entry.path();
        let path = path.clone()
            + "/"
            + entry_path
                .file_name()
                .ok_or(ErrorKind::InvalidData)?
                .to_str()
                .ok_or(ErrorKind::InvalidData)?;
        if entry_path.is_dir() {
            result.append(&mut list_file(&path)?)
        } else {
            result.push(path);
        }
    }
    Ok(result)
}
