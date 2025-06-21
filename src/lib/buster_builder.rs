use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use thiserror::Error;
use url::{ParseError, Url};

use crate::lib::{
    logger::traits::{BusterLogger, NullLogger},
    progress_handler::traits::ProgressHandler,
};

use super::buster::Buster;

const DEFAULT_THREADS_NUMBER: usize = 50;
const DEFAULT_RECURSIVE_MODE: usize = 0;
const DEFAULT_TIMEOUT: usize = 5;

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
}

pub struct BusterBuilder<P>
where
    P: ProgressHandler + Send + Sync + Default,
{
    threads: Option<usize>,
    recursion: Option<usize>,
    timeout: Option<usize>,
    wordlist: Option<PathBuf>,
    uri: Option<Url>,
    error: Option<BuilderError>,
    total_progress_handler: Option<Arc<P>>,
    current_progress_handler: Option<Arc<P>>,
    logger: Option<Arc<BusterLogger>>,
}

impl<P> BusterBuilder<P>
where
    P: ProgressHandler + Sync + Send + Default,
{
    pub fn new() -> Self {
        BusterBuilder {
            threads: None,
            recursion: None,
            wordlist: None,
            uri: None,
            error: None,
            total_progress_handler: None,
            current_progress_handler: None,
            logger: None,
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

    pub fn total_progress_handler(mut self, tpg: Arc<P>) -> Self {
        self.total_progress_handler = Some(tpg);
        self
    }

    pub fn current_progress_handler(mut self, cpg: Arc<P>) -> Self {
        self.current_progress_handler = Some(cpg);
        self
    }

    pub fn with_logger(mut self, logger: Arc<BusterLogger>) -> Self {
        self.logger = Some(logger);
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

    pub fn build(&self) -> Result<Buster<P>, BuilderError> {
        if let Some(err) = &self.error {
            return Err(err.clone());
        }

        let uri = self
            .uri
            .as_ref()
            .ok_or(BuilderError::HostNotSpecified)?
            .to_owned();

        let threads = self.threads.unwrap_or(DEFAULT_THREADS_NUMBER);
        let recursion_depth = self.recursion.unwrap_or(DEFAULT_RECURSIVE_MODE);
        let timeout = self.timeout.unwrap_or(DEFAULT_TIMEOUT);

        let total_progress_handler: Arc<P> = match self.total_progress_handler.as_ref() {
            Some(tpg) => Arc::clone(tpg),
            None => Arc::new(P::default()),
        };

        let current_progress_handler: Arc<P> = match self.current_progress_handler.as_ref() {
            Some(cpg) => Arc::clone(cpg),
            None => Arc::new(P::default()),
        };

        let logger: Arc<BusterLogger> = match self.logger.as_ref() {
            Some(log) => Arc::clone(log),
            None => Arc::new(BusterLogger::NullLogger(NullLogger::default())),
        };

        let wordlist = self
            .wordlist
            .as_ref()
            .ok_or(BuilderError::WordlistNotSpecified)?
            .to_owned();

        Ok(Buster::new(
            threads,
            recursion_depth,
            timeout,
            wordlist,
            uri,
            total_progress_handler,
            current_progress_handler,
            logger,
        ))
    }
}

impl<T: ProgressHandler + Send + Sync + Default> Default for BusterBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}
