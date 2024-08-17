use std::fs;
use std::path::Path;

pub fn list_dir(path: &String) -> Option<Vec<String>> {
    let mut result: Vec<String> = Vec::new();
    for entry in fs::read_dir(&Path::new(path)).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        result.push(path.file_name()?.to_str()?.into());
    }
    Some(result)
}

pub fn exists(path: &String) -> bool {
    let file = Path::new(&path);
    file.exists()
}
