use std::{fs, io, path::PathBuf};

use atomic_write_file::AtomicWriteFile;
use directories::ProjectDirs;

use crate::constants::{
    CONFIG_FILE_NAME, DEFAULT_HEIGHT, MAX_HEIGHT, MIN_HEIGHT, PROJECT_APPLICATION,
    PROJECT_ORGANIZATION, PROJECT_QUALIFIER,
};
use crate::storage::commit_text;

const HEIGHT_PREFIX: &str = "height:";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub default_height: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_height: DEFAULT_HEIGHT,
        }
    }
}

impl Config {
    pub fn load() -> io::Result<Self> {
        let Some(path) = path() else {
            return Ok(Self::default());
        };
        let source = match fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };
        Ok(Self::parse(&source))
    }

    pub fn save(&self) -> io::Result<()> {
        let Some(path) = path() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let source = format!("{HEIGHT_PREFIX}{}\n", self.default_height);
        let file = AtomicWriteFile::open(path)?;
        commit_text(file, &source)
    }

    fn parse(source: &str) -> Self {
        let mut config = Self::default();
        for line in source.lines() {
            if let Some(value) = line.strip_prefix(HEIGHT_PREFIX)
                && let Ok(value) = value.parse::<u16>()
                && (MIN_HEIGHT..=MAX_HEIGHT).contains(&value)
            {
                config.default_height = value;
            }
        }
        config
    }
}

fn path() -> Option<PathBuf> {
    ProjectDirs::from(PROJECT_QUALIFIER, PROJECT_ORGANIZATION, PROJECT_APPLICATION)
        .map(|directories| directories.config_dir().join(CONFIG_FILE_NAME))
}
