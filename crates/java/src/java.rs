use std::fs::{File, exists};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

use super::java_version::JavaVersion;

pub enum JavaInstallationError {
    JavaExecutableNotFound,
    ReleaseFileInvalid,
    IOError(std::io::Error),
}

impl From<std::io::Error> for JavaInstallationError {
    fn from(err: std::io::Error) -> Self {
        JavaInstallationError::IOError(err)
    }
}

pub struct JavaInstallation {
    /// path of java home
    path: String,
    version: JavaVersion,
}

impl JavaInstallation {
    pub fn new(path: String) -> Result<Self, JavaInstallationError> {
        Ok(JavaInstallation {
            path: path.clone(),
            version: Self::i_get_version(&path)?,
        })
    }

    pub fn get_java_path(&self) -> Result<String, JavaInstallationError> {
        Self::i_get_java_path(&self.path)
    }

    fn i_get_java_path(path: &str) -> Result<String, JavaInstallationError> {
        let path = format!("{}/bin/java", path);
        if exists(&path)? {
            Ok(path)
        } else if exists(format!("{}.exe", &path))? {
            Ok(format!("{}.exe", &path))
        } else {
            Err(JavaInstallationError::JavaExecutableNotFound)
        }
    }

    fn i_get_version(path: &str) -> Result<JavaVersion, JavaInstallationError> {
        // try to read release file
        let rel_path = format!("{}/release", &path);
        if exists(&rel_path)? {
            let file = File::open(Path::new(&rel_path))?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;
                if line.starts_with("JAVA_VERSION=") {
                    let version_str = line.trim_start_matches("JAVA_VERSION=").trim_matches('"');
                    return Ok(JavaVersion::from(version_str));
                }
            }
        }

        // get the result of java -version
        let java_path = Self::i_get_java_path(path)?;

        // the output is in stderr
        let output = Command::new(java_path).arg("-version").output()?.stderr;

        let mut ver_vec = Vec::new();
        let mut state = false;
        for c in output {
            if c == b'"' && state == false {
                state = true;
            } else if c == b'"' && state == true {
                break;
            } else if state == true {
                ver_vec.push(c);
            }
        }

        let ver_str = String::from_utf8(ver_vec).unwrap();
        Ok(JavaVersion::from(ver_str.as_str()))
    }
}
