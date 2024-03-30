use log::warn;
use std::fs;
use std::path::Path;

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