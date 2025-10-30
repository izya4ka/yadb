use anyhow::Result;
use console::style;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread::{self, ScopedJoinHandle};
use std::time::Duration;
use std::{fs::File, path::PathBuf};
use thiserror::Error;
use ureq::Agent;
use url::Url;

use crate::lib::buster_messages::{BusterMessage, ProgressChangeMessage, ProgressMessage};
use crate::lib::logger::traits::LogLevel;

#[derive(Error, Debug, Clone)]
pub enum BusterError {
    #[error("Request error: {0}")]
    RequestError(String),
}

#[derive(Debug)]
pub struct Buster {
    threads: usize,
    recursion_depth: usize,
    wordlist_path: PathBuf,
    message_sender: Arc<Sender<BusterMessage>>,
    uri: Url,
    timeout: usize,
}

impl Buster {
    pub fn new(
        threads: usize,
        recursion_depth: usize,
        timeout: usize,
        wordlist: PathBuf,
        uri: Url,
        message_sender: Arc<Sender<BusterMessage>>,
    ) -> Buster {
        Buster {
            threads,
            recursion_depth,
            wordlist_path: wordlist,
            message_sender,
            uri,
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

            self.message_sender
                .send(BusterMessage::set_total_size(progress_len))
                .expect("SENDER ERROR");

            self.message_sender
                .send(BusterMessage::set_current_size(progress_len))
                .expect("SENDER ERROR");

            let urls_result = self.execute(url, lines)?;

            progress_len += urls_result.len() * lines_len;
            urls_vec.extend(urls_result);
        }

        self.message_sender
            .send(BusterMessage::finish_current())
            .expect("SENDER ERROR");
        self.message_sender
            .send(BusterMessage::finish_total())
            .expect("SENDER ERROR");
        Ok(())
    }

    pub fn execute(&self, url: Url, lines: Arc<Vec<String>>) -> Result<Vec<Url>> {
        let slice_size = lines.len() / self.threads;

        let lines_arc = lines.clone();

        let mut result: Vec<Url> = Vec::new();

        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(self.timeout.try_into().unwrap())))
            .http_status_as_error(false)
            .build()
            .into();
        let client = Arc::new(agent);

        thread::scope(|s| {
            let mut threads: Vec<ScopedJoinHandle<Result<Vec<Url>, BusterError>>> = Vec::new();

            for thr in 0..self.threads {
                let words = lines_arc.clone();

                let message_sender = self.message_sender.clone();

                let client_cloned = client.clone();
                let url = url.clone();

                let threads_num = self.threads;

                threads.push(s.spawn(move || {
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
                                    // cpb.println(format!("GET {url} -> {}", style(status).cyan()));
                                    message_sender
                                        .send(BusterMessage::Progress(ProgressMessage::Current(
                                            ProgressChangeMessage::Print(format!(
                                                "GET {url} -> {}",
                                                style(status).cyan()
                                            )),
                                        )))
                                        .expect("SENDER ERROR");

                                    // logger.log(LogLevel::INFO, format!("{url} -> {status}"));
                                    message_sender
                                        .send(BusterMessage::Log(
                                            LogLevel::INFO,
                                            format!("{url} -> {status}"),
                                        ))
                                        .expect("SENDER ERROR");

                                    result.push(Url::parse(&url).unwrap());
                                } else {
                                    // cpb.set_message(format!("GET {url} -> {}", style(status).red()));
                                    message_sender
                                        .send(BusterMessage::Progress(ProgressMessage::Current(
                                            ProgressChangeMessage::SetMessage(format!(
                                                "GET {url} -> {}",
                                                style(status).red()
                                            )),
                                        )))
                                        .expect("SENDER ERROR");
                                }
                            }
                            Err(e) => {
                                // cpb.println(format!(
                                //     "Error while sending request to {}: {e}",
                                //     style(&url).red()
                                // ));
                                message_sender
                                    .send(BusterMessage::Progress(ProgressMessage::Current(
                                        ProgressChangeMessage::Print(format!(
                                            "Error while sending request to {}: {e}",
                                            style(&url).red()
                                        )),
                                    )))
                                    .expect("SENDER ERROR")
                            }
                        }
                        // cpb.advance();
                        // tpb.advance();

                        message_sender
                            .send(BusterMessage::advance_current())
                            .expect("SENDER ERROR");
                    
                        message_sender
                            .send(BusterMessage::advance_total())
                            .expect("SENDER ERROR");
                    }
                    Ok(result)
                }));
            }

            for thread in threads {
                match thread.join() {
                    Ok(Ok(res)) => {
                        result.extend(res);
                    }

                    Ok(Err(err)) => self
                        .message_sender
                        .send(BusterMessage::log(LogLevel::ERROR, err.to_string()))
                        .expect("SENDER ERROR"),
                    Err(err) => self
                        .message_sender
                        .send(BusterMessage::log(
                            LogLevel::CRITICAL,
                            format!("Panic in thread: {err:?}"),
                        ))
                        .expect("SENDER ERROR"),
                }
            }
        });

        Ok(result)
    }
}
