use anyhow::Result;
use console::style;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{fs::File, path::PathBuf};
use thiserror::Error;
use ureq::Agent;
use url::Url;

use crate::lib::logger::traits::{BusterLogger, LogLevel};
use crate::lib::progress_handler::traits::ProgressHandler;

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
    recursion_depth: usize,
    wordlist_path: PathBuf,
    uri: Url,
    total_progress_handler: Arc<T>,
    current_progress_handler: Arc<T>,
    logger: Arc<BusterLogger>,
    timeout: usize,
}

impl<Progress> Buster<Progress>
where
    Progress: ProgressHandler + Default + Send + Sync + 'static,
{
    pub fn new(
        threads: usize,
        recursion_depth: usize,
        timeout: usize,
        wordlist: PathBuf,
        uri: Url,
        total_progress_handler: Arc<Progress>,
        current_progress_handler: Arc<Progress>,
        logger: Arc<BusterLogger>,
    ) -> Buster<Progress> {
        Buster {
            threads,
            recursion_depth,
            wordlist_path: wordlist,
            uri,
            total_progress_handler,
            current_progress_handler,
            logger,
            timeout,
        }
    }

    pub fn run(&self) -> Result<()> {
        let mut urls_vec: Vec<Url> = Vec::new();
        urls_vec.push(self.uri.clone());
        let file = File::open(&self.wordlist_path)?;
        let lines: Arc<Vec<String>> =
            Arc::new(BufReader::new(file).lines().map_while(Result::ok).collect());
        let lines_len = lines.len();
        let mut progress_len = lines_len;
        let path_len_start = self.uri.path_segments().unwrap().collect::<Vec<_>>().len();

        while let Some(url) = urls_vec.pop() {
            if url.path_segments().unwrap().collect::<Vec<_>>().len() - path_len_start
                > self.recursion_depth
            {
                continue;
            }

            let lines = lines.clone();
            self.current_progress_handler.set_size(progress_len);
            self.total_progress_handler.set_size(progress_len);
            let urls_result = self.execute(url, lines)?;

            progress_len += urls_result.len() * lines_len;
            urls_vec.extend(urls_result);
        }

        self.current_progress_handler.finish();
        self.total_progress_handler.finish();
        Ok(())
    }

    pub fn execute(&self, url: Url, lines: Arc<Vec<String>>) -> Result<Vec<Url>> {
        let slice_size = lines.len() / self.threads;

        let lines_arc = lines.clone();

        let mut result: Vec<Url> = Vec::new();

        let mut threads: Vec<JoinHandle<Result<Vec<Url>, BusterError>>> = Vec::new();

        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(self.timeout.try_into().unwrap())))
            .http_status_as_error(false)
            .build()
            .into();
        let client = Arc::new(agent);

        for thr in 0..self.threads {
            let words = lines_arc.clone();

            let tpb = self.total_progress_handler.clone();
            let cpb = self.current_progress_handler.clone();

            let client_cloned = client.clone();
            let url = url.clone();

            let logger = self.logger.clone();
            let threads_num = self.threads;

            threads.push(thread::spawn(move || {
                let words = words.clone();
                let words_slice = if thr != threads_num - 1 {
                    &words[slice_size * thr..slice_size * thr + slice_size]
                } else {
                    &words[slice_size * thr..]
                };

                let mut result: Vec<Url> = Vec::new();

                for word in words_slice {
                    let url = if url.to_string().ends_with("/") {
                        format!("{url}{word}/")
                    } else {
                        format!("{url}/{word}/")
                    };

                    match client_cloned.get(&url).call() {
                        Ok(res) => {
                            let status = res.status().as_u16();
                            if status != 404 {
                                cpb.println(format!("GET {url} -> {}", style(status).cyan()));
                                logger.log(LogLevel::INFO, format!("{url} -> {status}"));
                                result.push(Url::parse(&url).unwrap());
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
                Ok(result)
            }));
        }

        for thread in threads {
            match thread.join() {
                Ok(Ok(res)) => {
                    result.extend(res);
                }

                Ok(Err(err)) => self.logger.log(LogLevel::ERROR, err.to_string()),
                Err(err) => self
                    .logger
                    .log(LogLevel::CRITICAL, format!("Panic in thread: {err:?}")),
            }
        }

        Ok(result)
    }
}
