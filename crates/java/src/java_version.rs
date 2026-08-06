#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaVersion {
    feature: u8,
    interim: u8,
    update: u32,
    patch: u32,
}

impl From<&str> for JavaVersion {
    fn from(version_str: &str) -> Self {
        let vec = version_str.split('.').collect::<Vec<&str>>();
        if version_str.starts_with("1.") {
            // <= Java 8
            let vec2 = vec[2].split("_").collect::<Vec<&str>>();
            JavaVersion {
                feature: vec[1].parse().unwrap_or(0),
                interim: vec2[0].parse().unwrap_or(0),
                update: vec2[1].parse().unwrap_or(0),
                patch: 0,
            }
        } else {
            // >= Java 9
            let mut patch = 0;
            if vec.len() == 4 {
                patch = vec[3].parse().unwrap_or(0);
            }
            JavaVersion {
                feature: vec[0].parse().unwrap_or(0),
                interim: vec[1].parse().unwrap_or(0),
                update: vec[2].parse().unwrap_or(0),
                patch,
            }
        }
    }
}

impl std::fmt::Display for JavaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.feature <= 8 {
            write!(f, "1.{}.{}.{}", self.feature, self.interim, self.update)
        } else {
            if self.patch == 0 {
                write!(f, "{}.{}.{}", self.feature, self.interim, self.update)
            } else {
                write!(
                    f,
                    "{}.{}.{}.{}",
                    self.feature, self.interim, self.update, self.patch
                )
            }
        }
    }
}

impl JavaVersion {
    pub fn check_minimum_version(&self, min_version: &JavaVersion) -> bool {
        self >= min_version
    }

    pub fn check_maximum_version(&self, max_version: &JavaVersion) -> bool {
        self <= max_version
    }
}
