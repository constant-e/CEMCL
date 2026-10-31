use std::fs::{File, exists};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;

use super::java_version::JavaVersion;

#[derive(Debug)]
pub enum JavaInstallationError {
    JavaExecutableNotFound,
    PathIsEmpty,
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
    path: PathBuf,
    version: JavaVersion,
}

impl JavaInstallation {
    pub fn new(path: PathBuf) -> Result<Self, JavaInstallationError> {
        if path.as_os_str().is_empty() {
            return Err(JavaInstallationError::PathIsEmpty);
        }
        Ok(JavaInstallation {
            version: Self::i_get_version(&path)?,
            path,
        })
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }

    pub fn get_version(&self) -> &JavaVersion {
        &self.version
    }

    pub fn get_java_path(&self) -> Result<PathBuf, JavaInstallationError> {
        Self::i_get_java_path(&self.path)
    }

    fn i_get_java_path(path: &Path) -> Result<PathBuf, JavaInstallationError> {
        let java_path = path.join("bin").join("java");
        if exists(&java_path)? {
            Ok(java_path)
        } else {
            let java_exe = java_path.with_extension("exe");
            if exists(&java_exe)? {
                Ok(java_exe)
            } else {
                Err(JavaInstallationError::JavaExecutableNotFound)
            }
        }
    }

    fn i_get_version(path: &Path) -> Result<JavaVersion, JavaInstallationError> {
        // try to read release file
        let rel_path = path.join("release");
        if exists(&rel_path)? {
            let file = File::open(&rel_path)?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;
                if line.starts_with("JAVA_VERSION=") {
                    let version_str = line.trim_start_matches("JAVA_VERSION=").trim_matches('"');
                    return Ok(JavaVersion::from(version_str));
                }
            }

            return Err(JavaInstallationError::ReleaseFileInvalid);
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
