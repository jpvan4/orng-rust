use clap::Parser;
use obfstr::obfstr;


use orng_rust::{Error, Result, Stratum, Worker};
use std::{
    num::NonZeroUsize,
    time::{Duration, Instant},
};

const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Parser)]
struct Args {
    #[arg(short = 'o', long, default_value = "pool.hashvault.pro:443")]
    url: String,
    #[arg(
        short,
        long,
        default_value = "44qARb3o5kWimeStvm9g4r5kTCMSZio8SEWDcEy9HKnnXg6iQns7Mqi4SrrSNZV6mG1YQWqRgr5Lph1BxfQFK8Kz8hMidXR"
    )]
    user: String,
    #[arg(short, long, default_value = "x")]
    pass: String,
    #[arg(short, long, default_value_t = all_threads())]
    threads: NonZeroUsize,
    #[arg(long)]
    light: bool,
}

fn all_threads() -> NonZeroUsize {
    std::thread::available_parallelism().unwrap()
}

fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::DEBUG)
        .with_file(false)
        .with_line_number(false)
        .init();

    let Args {
        url,
        user,
        pass,
        light,
        threads,
    } = Args::parse();

    let mut stratum = Stratum::login(&url, &user, &pass)?;
    let first_job = loop {
        match stratum.try_recv_job() {
            Ok(job) => break job,
            Err(Error::Channel(ref msg)) if msg == "Channel is empty" => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            Err(e) => return Err(Error::Stratum(format!("Failed to get first job: {}", e))),
        }
    };
    let worker = Worker::init(first_job, threads, light)?;
    let mut timer = Instant::now();

    loop {
        if let Ok(job) = stratum.try_recv_job() {
            worker.update_job(job);
        }

        if let Some(share) = worker.try_recv_share() {
            stratum.submit(share)?;
        }

        if timer.elapsed() >= KEEP_ALIVE_INTERVAL {
            stratum.keep_alive()?;
            timer = Instant::now();
        }
    }
}

fn main() {
    if let Err(e) = run() {
        tracing::error!("Application error: {}", e);
        std::process::exit(1);
    }
}
