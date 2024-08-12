use log::warn;
use std::io;
use std::fs::{self, ReadDir};
use std::path::Path;

pub fn list_dir(path: &String) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for entry in fs::read_dir(&Path::new(path)).expect("Err") {
        let entry = entry.expect("Err");
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        result.push(path.file_name().expect("Err").to_str().expect("Err").into());
    }
    result
}

pub fn exists(path: &String) -> bool {
    let file = Path::new(&path);
    file.exists()
}

pub fn open_file(path: &String) -> String {
    let f = fs::read_to_string(path);
    match f {
        Ok(result) => return result,
        Err(error) => {
            warn!("file_tools: Unable to read {path}");
            return String::new();
        }
    }
}

pub fn save_file(path: &String, contents: &String) -> bool {
    fs::write(path, contents)
        .is_ok()
}
