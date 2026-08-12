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
            // <= Java 8, e.g. "1.8.0_202" or "1.8.0"
            let feature = vec.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let (interim, update) = if vec.len() > 2 {
                let vec2 = vec[2].split('_').collect::<Vec<&str>>();
                let interim = vec2[0].parse().unwrap_or(0);
                let update = vec2.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                (interim, update)
            } else {
                (0, 0)
            };
            JavaVersion {
                feature,
                interim,
                update,
                patch: 0,
            }
        } else {
            // >= Java 9, e.g. "21.0.1" or just "21"
            let feature = vec.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            let interim = vec.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let update = vec.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
            let patch = vec.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
            JavaVersion {
                feature,
                interim,
                update,
                patch,
            }
        }
    }
}

impl std::fmt::Display for JavaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.feature <= 8 {
            write!(f, "1.{}.{}_{}", self.feature, self.interim, self.update)
        } else if self.patch == 0 {
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

impl JavaVersion {
    pub fn check_minimum_version(&self, min_version: &JavaVersion) -> bool {
        self >= min_version
    }

    pub fn check_maximum_version(&self, max_version: &JavaVersion) -> bool {
        self <= max_version
    }
}
