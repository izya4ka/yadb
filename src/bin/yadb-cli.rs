use std::{
    fmt::Write,
    sync::{Mutex, mpsc},
    thread,
};

use clap::Parser;
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressState, ProgressStyle};
use yadb::lib::{
    logger::{
        file_logger::FileLogger,
        traits::{NullLogger, WorkerLogger},
    },
    util,
    worker::{
        builder::WorkerBuilder,
        messages::{ProgressChangeMessage, ProgressMessage, WorkerMessage},
    },
};

#[derive(Parser)]
#[command(name = "yadb-cli")]
#[command(version)]
#[command(about = "Yet Another Directory Buster")]
#[command(long_about = None)]
struct Args {
    /// Number of threads
    #[arg(short, long, default_value_t = 50)]
    threads: usize,

    /// Timeout of request in seconds
    #[arg(long, default_value_t = 5)]
    timeout: usize,

    /// Recursivly parse directories and files (recursion depth)
    #[arg(short, long, default_value_t = 0)]
    recursion: usize,

    /// Path to wordlist
    #[arg(short, long)]
    wordlist: String,

    /// Target URI
    #[arg(short, long)]
    uri: String,

    /// Output file
    #[arg(short, long)]
    output: Option<String>,
}
fn main() {
    let args: Args = Args::parse();

    util::print_logo();
    println!("Threads: {}", style(args.threads.to_string()).cyan());
    println!(
        "Recursion depth: {}",
        style(args.recursion.to_string()).cyan()
    );
    println!(
        "Timeout: {} seconds",
        style(args.timeout.to_string()).cyan()
    );
    println!("Wordlist path: {}", style(args.wordlist.to_string()).cyan());
    println!("Target: {}", style(args.uri.to_string()).cyan());
    if let Some(output) = args.output.as_ref() {
        println!("Output: {}\n", style(output.to_string()).cyan());
    }

    let m = MultiProgress::new();

    let cpb = m.add(ProgressBar::no_length());
    cpb.set_style(
        ProgressStyle::with_template("{spinner:.green} {prefix:.bold.dim} {wide_msg}").unwrap(),
    );

    let tpb = m.add(ProgressBar::no_length());
    tpb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7} ({eta})",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"),
    );

    let logger = if let Some(output) = args.output {
        match FileLogger::new(output) {
            Ok(log) => WorkerLogger::FileLogger(Mutex::new(log)),
            Err(err) => {
                println!("Error: {err}");
                return;
            }
        }
    } else {
        WorkerLogger::NullLogger(NullLogger::default())
    };

    let (tx, rx) = mpsc::channel::<WorkerMessage>();

    let worker = WorkerBuilder::default()
        .recursive(args.recursion)
        .threads(args.threads)
        .timeout(args.timeout)
        .uri(&args.uri)
        .message_sender(tx.into())
        .wordlist(&args.wordlist)
        .build();

    match worker {
        Ok(buster) => {
            thread::spawn(move || buster.run());

            for msg in rx {
                match msg {
                    WorkerMessage::Progress(progress_message) => match progress_message {
                        ProgressMessage::Current(progress_change_message) => {
                            match progress_change_message {
                                ProgressChangeMessage::SetMessage(str) => cpb.set_message(str),
                                ProgressChangeMessage::SetSize(size) => {
                                    cpb.set_length(size.try_into().unwrap())
                                }
                                ProgressChangeMessage::Start(size) => {
                                    cpb.reset();
                                    cpb.set_length(size.try_into().unwrap());
                                }
                                ProgressChangeMessage::Advance => cpb.inc(1),
                                ProgressChangeMessage::Print(str) => cpb.println(str),
                                ProgressChangeMessage::Finish => cpb.finish(),
                            }
                        }
                        ProgressMessage::Total(progress_change_message) => {
                            match progress_change_message {
                                ProgressChangeMessage::SetMessage(str) => tpb.set_message(str),
                                ProgressChangeMessage::SetSize(size) => {
                                    tpb.set_length(size.try_into().unwrap())
                                }
                                ProgressChangeMessage::Start(size) => {
                                    tpb.reset();
                                    tpb.set_length(size.try_into().unwrap());
                                }
                                ProgressChangeMessage::Advance => tpb.inc(1),
                                ProgressChangeMessage::Print(str) => tpb.println(str),
                                ProgressChangeMessage::Finish => tpb.finish(),
                            }
                        }
                    },
                    WorkerMessage::Log(log_level, str) => {
                        logger.log(log_level, str);
                    }
                }
            }
        }

        Err(err) => println!("Error: {err}"),
    }
}
