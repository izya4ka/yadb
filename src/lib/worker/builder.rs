use std::{
    path::PathBuf,
    sync::{Arc, mpsc::Sender},
};

use anyhow::Result;
use thiserror::Error;
use url::{ParseError, Url};

use crate::lib::worker::{messages::WorkerMessage, unit::Worker};

pub const DEFAULT_THREADS_NUMBER: usize = 50;
pub const DEFAULT_RECURSIVE_MODE: usize = 0;
pub const DEFAULT_TIMEOUT: usize = 5;

#[derive(Error, Debug, Clone)]
pub enum BuilderError {
    #[error("Can't parse host: {0}")]
    HostParseError(#[from] ParseError),

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

    #[error("Sender channel not specified")]
    SenderChannelNotSpecified,
}

#[derive(Debug, Clone)]
pub struct WorkerBuilder {
    pub threads: Option<usize>,
    pub recursion: Option<usize>,
    pub timeout: Option<usize>,
    pub wordlist: Option<PathBuf>,
    pub uri: Option<Url>,
    error: Option<BuilderError>,
    message_sender: Option<Arc<Sender<WorkerMessage>>>,
}

impl WorkerBuilder {
    pub fn new() -> Self {
        WorkerBuilder {
            threads: None,
            recursion: None,
            wordlist: None,
            uri: None,
            error: None,
            message_sender: None,
            timeout: None,
        }
    }

    pub fn threads(mut self, threads: usize) -> Self {
        if self.error.is_some() {
            return self;
        }

        self.threads = Some(threads);
        self
    }

    pub fn message_sender(mut self, sender: Arc<Sender<WorkerMessage>>) -> Self {
        self.message_sender = Some(sender);
        self
    }

    pub fn recursive(mut self, recursive: usize) -> Self {
        if self.error.is_some() {
            return self;
        }

        self.recursion = Some(recursive);
        self
    }

    pub fn timeout(mut self, timeout: usize) -> Self {
        if self.error.is_some() {
            return self;
        }

        self.timeout = Some(timeout);
        self
    }

    pub fn wordlist(mut self, wordlist_path: &str) -> Self {
        if self.error.is_some() {
            return self;
        }

        let path: PathBuf = PathBuf::from(wordlist_path);

        if !path.exists() {
            self.error = Some(BuilderError::FileNotFound(wordlist_path.to_string()));
            return self;
        }

        if !path.is_file() {
            self.error = Some(BuilderError::NotAFile(wordlist_path.to_string()));
            return self;
        }

        if path.to_str().is_none() {
            self.error = Some(BuilderError::InvalidFilePath);
            return self;
        }

        self.wordlist = Some(path);
        self
    }

    pub fn uri(mut self, uri: &str) -> Self {
        if self.error.is_some() {
            return self;
        }

        let parsed_uri = match Url::parse(uri) {
            Ok(url) => url,
            Err(err) => {
                self.error = Some(BuilderError::HostParseError(err));
                return self;
            }
        };

        self.uri = Some(parsed_uri);

        self
    }

    pub fn build(self) -> Result<Worker, BuilderError> {
        if let Some(err) = self.error {
            return Err(err);
        }

        let uri = self.uri.ok_or(BuilderError::HostNotSpecified)?;

        let threads = self.threads.unwrap_or(DEFAULT_THREADS_NUMBER);
        let recursion_depth = self.recursion.unwrap_or(DEFAULT_RECURSIVE_MODE);
        let timeout = self.timeout.unwrap_or(DEFAULT_TIMEOUT);

        let wordlist = self.wordlist.ok_or(BuilderError::WordlistNotSpecified)?;

        let message_sender = self
            .message_sender
            .ok_or(BuilderError::SenderChannelNotSpecified)?;

        Ok(Worker::new(
            threads,
            recursion_depth,
            timeout,
            wordlist,
            uri,
            message_sender,
        ))
    }
}

impl Default for WorkerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
