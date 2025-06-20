use console::style;
use reqwest::blocking::Client;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::{fs::File, path::PathBuf};
use thiserror::Error;
use url::Url;

use crate::ProgressHandler;
use crate::lib::logger::traits::{BusterLogger, LogLevel};

#[derive(Error, Debug, Clone)]
pub enum BusterError {
    #[error("Request error: {0}")]
    RequestError(String),
}

pub struct Buster<T>
where
    T: ProgressHandler + Send + Sync + 'static,
{
    threads: usize,
    recursive: bool,
    wordlist_path: PathBuf,
    uri: Url,
    total_progress_handler: Arc<T>,
    current_progress_handler: Arc<T>,
    logger: Arc<BusterLogger>,
}

impl<Progress> Buster<Progress>
where
    Progress: ProgressHandler + Default + Send + Sync + 'static,
{
    pub fn new(
        threads: usize,
        recursive: bool,
        wordlist: PathBuf,
        uri: Url,
        total_progress_handler: Arc<Progress>,
        current_progress_handler: Arc<Progress>,
        logger: Arc<BusterLogger>,
    ) -> Buster<Progress> {
        Buster {
            threads,
            recursive,
            wordlist_path: wordlist,
            uri,
            total_progress_handler,
            current_progress_handler,
            logger,
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(&self.wordlist_path)?;
        let lines: Vec<String> = BufReader::new(file).lines().map_while(Result::ok).collect();

        let slice_size = lines.len() / self.threads;

        {
            self.current_progress_handler.start(lines.len());
            self.total_progress_handler.start(lines.len());
        }

        let lines_arc = Arc::new(lines);

        let mut threads: Vec<JoinHandle<Result<(), BusterError>>> = Vec::new();

        let client = Arc::new(Client::new());

        for thr in 0..self.threads {
            let words = lines_arc.clone();

            let tpb = self.total_progress_handler.clone();
            let cpb = self.current_progress_handler.clone();

            let client_cloned = client.clone();
            let url = self.uri.clone();

            let logger = self.logger.clone();

            let threads_num = self.threads;

            threads.push(thread::spawn(move || {
                let words = words.clone();
                let words_slice = if thr != threads_num - 1 {
                    &words[slice_size * thr..slice_size * thr + slice_size]
                } else {
                    &words[slice_size * thr..]
                };

                for word in words_slice {
                    let url = format!("{url}{word}");

                    match client_cloned.get(&url).send() {
                        Ok(res) => {
                            let status = res.status().as_u16();

                            if status != 404 {
                                cpb.println(format!("GET {url} -> {}", style(status).cyan()));
                                logger.log(LogLevel::INFO, format!("{url} -> {status}"));
                            } else {
                                cpb.set_message(format!("GET {url} -> {}", style(status).red()));
                            }
                        }
                        Err(e) => {
                            cpb.println(format!(
                                "Error while sending request to {}: {e}",
                                style(&url).red()
                            ));
                        }
                    }
                    cpb.advance();
                    tpb.advance();
                }
                Ok(())
            }));
        }

        for thread in threads {
            match thread.join() {
                Ok(Err(err)) => self.logger.log(LogLevel::ERROR, err.to_string()),
                Ok(Ok(())) => (),
                Err(err) => self
                    .logger
                    .log(LogLevel::CRITICAL, format!("Panic in thread: {err:?}")),
            }
        }

        self.current_progress_handler.finish();
        self.total_progress_handler.finish();

        Ok(())
    }
}
