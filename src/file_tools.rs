use std::fs;

pub fn open_file(path: &String) -> String {
    fs::read_to_string(path)
        .expect(format!("[Error] file_tools: Unable to read {path}").as_str())
}

pub fn save_file(path: &String, contents: &String) -> bool {
    fs::write(path, contents)
        .is_ok()
}