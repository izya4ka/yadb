use std::{fmt::Write, sync::Arc};

use clap::Parser;
use console::style;
use flexi_logger::{FileSpec, Logger};
use indicatif::{MultiProgress, ProgressBar, ProgressState, ProgressStyle};
use log::error;
use yadb::{
    CliProgress,
    lib::{
        buster::Buster,
        buster_builder::{BuilderError, BusterBuilder},
        util,
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

    /// Recursivly parse directories and files (TODO!)
    #[arg(short, long)]
    recursive: bool,

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
    println!("Recursive: {}", style(args.recursive.to_string()).cyan());
    println!("Wordlist path: {}", style(args.wordlist.to_string()).cyan());
    println!("Target: {}", style(args.uri.to_string()).cyan());
    if let Some(output) = args.output.as_ref() {
        println!("Output: {}\n", style(output.to_string()).cyan());
    }

    let mut logger: Logger = Logger::try_with_str("info")
        .unwrap()
        .format(util::log_format);

    if let Some(path) = args.output
        && let Ok(filespec) = FileSpec::try_from(path)
    {
        logger = logger.log_to_file(filespec);
    }

    let logger = logger.start();
    if let Err(err) = logger {
        error!("Failed to init logger: {err}")
    }

    let m = MultiProgress::new();

    let cpb = m.add(ProgressBar::no_length());
    cpb.set_style(ProgressStyle::with_template("{prefix:.bold.dim} {wide_msg}").unwrap());

    let tpb = m.add(ProgressBar::no_length());
    tpb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7} ({eta})",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"),
    );

    let total_progress_handler: CliProgress = CliProgress { pb: tpb };
    let current_progress_handler = CliProgress { pb: cpb };

    let builder: BusterBuilder<CliProgress> = BusterBuilder::new();
    let buster: Result<Buster<CliProgress>, BuilderError> = builder
        .recursive(args.recursive)
        .threads(args.threads)
        .uri(&args.uri)
        .wordlist(&args.wordlist)
        .total_progress_handler(Arc::new(total_progress_handler))
        .current_progress_handler(Arc::new(current_progress_handler))
        .build();

    if let Ok(buster) = buster {
        let _ = buster.run();
    }
}
