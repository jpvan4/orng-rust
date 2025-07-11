use clap::Parser;
use orng_rust::{Stratum, Worker};
use std::{
    io,
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

fn main() -> io::Result<()> {
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
    let worker = Worker::init(stratum.try_recv_job().unwrap(), threads, light);
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
