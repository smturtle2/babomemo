use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use atomic_write_file::AtomicWriteFile;

use crate::document::{Document, ParseError};

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Parse(ParseError),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Parse(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<io::Error> for LoadError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ParseError> for LoadError {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}

pub fn load(path: &Path) -> Result<(Document, bool), LoadError> {
    match fs::read_to_string(path) {
        Ok(source) => Ok((Document::parse(&source)?, false)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok((Document::new(), true)),
        Err(error) => Err(error.into()),
    }
}

pub fn save(path: &Path, document: &Document) -> io::Result<()> {
    let file = AtomicWriteFile::open(path)?;
    commit_text(file, &document.serialize())
}

pub fn commit_text(mut file: AtomicWriteFile, source: &str) -> io::Result<()> {
    file.write_all(source.as_bytes())?;
    file.commit()
}

pub fn memo_path(directory: &Path, file_name: &str) -> PathBuf {
    directory.join(file_name)
}
