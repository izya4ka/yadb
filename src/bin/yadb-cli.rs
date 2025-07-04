use std::{
    fmt::Write,
    sync::{Arc, Mutex},
};

use clap::Parser;
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressState, ProgressStyle};
use yadb::lib::{
    buster_builder::BusterBuilder,
    logger::{
        file_logger::FileLogger,
        traits::{BusterLogger, NullLogger},
    },
    progress_handler::traits::CliProgress,
    util,
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

    let total_progress_handler = CliProgress { pb: tpb };
    let current_progress_handler = CliProgress { pb: cpb };

    let logger = if let Some(output) = args.output {
        match FileLogger::new(output) {
            Ok(log) => BusterLogger::FileLogger(Mutex::new(log)),
            Err(err) => {
                println!("Error: {err}");
                return;
            }
        }
    } else {
        BusterLogger::NullLogger(NullLogger::default())
    };

    let buster = BusterBuilder::new()
        .recursive(args.recursion)
        .threads(args.threads)
        .timeout(args.timeout)
        .uri(&args.uri)
        .wordlist(&args.wordlist)
        .total_progress_handler(Arc::new(total_progress_handler))
        .current_progress_handler(Arc::new(current_progress_handler))
        .with_logger(Arc::new(logger))
        .build();

    match buster {
        Ok(buster) => {
            let _ = buster.run();
        }
        Err(err) => println!("Error: {err}"),
    }
}
