use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub fn list_all(path: &String) -> Option<Vec<String>> {
    let mut result: Vec<String> = Vec::new();
    for entry in fs::read_dir(&Path::new(path)).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        result.push(path.file_name()?.to_str()?.into());
    }
    Some(result)
}

pub fn list_dir(path: &String) -> std::io::Result<Vec<String>> {
    let mut result: Vec<String> = Vec::new();
    for entry in fs::read_dir(&Path::new(path))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        result.push(path.file_name().ok_or(ErrorKind::InvalidData)?.to_str().ok_or(ErrorKind::InvalidData)?.into());
    }
    Ok(result)
}

pub fn list_file(path: &String) -> Option<Vec<String>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(&Path::new(path)).ok()? {
        let entry = entry.ok()?;
        let entry_path = entry.path();
        let path = path.clone() + "/" + entry_path.file_name()?.to_str()?;
        if entry_path.is_dir() {
            result.append(&mut list_file(&path)?)
        } else {
            result.push(path);
        }
    }
    Some(result)
}
