use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use log::info;
use thiserror::Error;
use url::Url;

use crate::ProgressHandler;

use super::buster::Buster;

const DEFAULT_THREADS_NUMBER: usize = 10;
const DEFAULT_RECURSIVE_MODE: bool = false;

#[derive(Error, Debug, Clone)]
pub enum BuilderError {
    #[error("Can't parse host from: {0}")]
    HostParseError(String),

    #[error("Host not specified")]
    HostNotSpecified,

    #[error("Port not specified")]
    PortNotSpecified,

    #[error("Wordlist not specified")]
    WordlistNotSpecified,

    #[error("Non-UTF8 file path")]
    InvalidFilePath,

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Not a file: {0}")]
    NotAFile(String),
}

pub struct BusterBuilder<T: ProgressHandler> {
    threads: Option<usize>,
    recursive: Option<bool>,
    wordlist: Option<PathBuf>,
    uri: Option<Url>,
    error: Option<BuilderError>,
    total_progress_handler: Option<Arc<T>>,
    current_progress_handler: Option<Arc<T>>,
}

impl<T: ProgressHandler + Default> BusterBuilder<T> {
    pub fn new() -> Self {
        BusterBuilder {
            threads: None,
            recursive: None,
            wordlist: None,
            uri: None,
            error: None,
            total_progress_handler: None,
            current_progress_handler: None,
        }
    }

    pub fn threads(mut self, threads: usize) -> Self {
        if self.error.is_some() {
            return self;
        }

        self.threads = Some(threads);
        self
    }

    pub fn recursive(mut self, recursive: bool) -> Self {
        if self.error.is_some() {
            return self;
        }

        self.recursive = Some(recursive);
        self
    }

    pub fn wordlist(mut self, wordlist_path: &str) -> Self {
        if self.error.is_some() {
            return self;
        }

        let path: PathBuf = PathBuf::from(wordlist_path);

        if !path.exists() {
            self.error = Some(BuilderError::FileNotFound(wordlist_path.to_string()))
        }

        if !path.is_file() {
            self.error = Some(BuilderError::NotAFile(wordlist_path.to_string()))
        }

        if path.to_str().is_none() {
            self.error = Some(BuilderError::InvalidFilePath);
        }

        self.wordlist = Some(path);
        self
    }

    pub fn total_progress_handler(mut self, tpg: Arc<T>) -> Self {
        self.total_progress_handler = Some(tpg);
        self
    }

    pub fn current_progress_handler(mut self, cpg: Arc<T>) -> Self {
        self.current_progress_handler = Some(cpg);
        self
    }

    pub fn uri(mut self, uri: &str) -> Self {
        if self.error.is_some() {
            return self;
        }

        let parsed_uri = match Url::parse(uri) {
            Ok(url) => url,
            Err(_) => {
                self.error = Some(BuilderError::HostParseError(uri.to_string()));
                return self;
            }
        };

        self.uri = Some(parsed_uri);

        self
    }

    pub fn build(&self) -> Result<Buster<T>, BuilderError> {
        if let Some(err) = &self.error {
            return Err(err.clone());
        }

        let uri = self
            .uri
            .as_ref()
            .ok_or(BuilderError::HostNotSpecified)?
            .to_owned();

        info!("Host: {uri}");

        let threads = self.threads.unwrap_or_else(|| {
            info!("Using default number of threads: {DEFAULT_THREADS_NUMBER}");
            DEFAULT_THREADS_NUMBER
        });

        let recursive = self.recursive.unwrap_or_else(|| {
            info!("Using default recursive mode: {DEFAULT_RECURSIVE_MODE}");
            DEFAULT_RECURSIVE_MODE
        });

        let total_progress_handler: Arc<T> = match self.total_progress_handler.as_ref() {
            Some(tpg) => tpg.to_owned(),
            None => Arc::new(T::default()),
        };

        let current_progress_handler: Arc<T> = match self.current_progress_handler.as_ref() {
            Some(tpg) => tpg.to_owned(),
            None => Arc::new(T::default()),
        };

        let wordlist = self
            .wordlist
            .as_ref()
            .ok_or(BuilderError::WordlistNotSpecified)?
            .to_owned();

        info!("Using wordlist from: {}", wordlist.to_string_lossy());

        Ok(Buster::new(
            threads,
            recursive,
            wordlist,
            uri,
            total_progress_handler,
            current_progress_handler,
        ))
    }
}

impl<T: ProgressHandler + Default> Default for BusterBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}
