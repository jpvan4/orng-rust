use crate::{job::Job, share::Share};
use core_affinity;
use randomx_rs::{RandomXCache, RandomXDataset, RandomXFlag, RandomXVM};
use std::{
    num::NonZeroUsize,
    sync::mpsc::{self, Receiver},
    thread,
    time::Instant,
};
// Import the specific types from watch crate
use watch::{channel, WatchSender};
pub struct Worker {
    share_rx: Receiver<Share>,
    job_tx: WatchSender<Job>,
}
impl Worker {
    #[tracing::instrument(skip(job))]
    pub fn init(job: Job, num_threads: NonZeroUsize, light: bool) -> Self {
        let (share_tx, share_rx) = mpsc::channel();
        let (job_tx, job_rx) = channel(job.clone());

        let mut flags = RandomXFlag::get_recommended_flags()
            | RandomXFlag::FLAG_JIT
            | RandomXFlag::FLAG_HARD_AES;
        if !light {
            flags |= RandomXFlag::FLAG_FULL_MEM;
        }

        let cores = core_affinity::get_core_ids().unwrap_or_default();
        let (hashrate_tx, hashrate_rx) = mpsc::channel();
        for i in 0..num_threads.get() {
            let core_id = cores.get(i % cores.len()).cloned();
            let share_tx = share_tx.clone();
            let mut job_rx = job_rx.clone();
            let hashrate_tx = hashrate_tx.clone();
            let thread_nonce_start = (i as u32) * (u32::MAX / num_threads.get() as u32);
            let thread_nonce_end = thread_nonce_start + (u32::MAX / num_threads.get() as u32);
            thread::spawn(move || {
                if let Some(core) = core_id {
                    core_affinity::set_for_current(core);
                    tracing::info!("Thread {i} pinned to core {:?}", core.id);
                }
                let mut cache = RandomXCache::new(flags, &job_rx.get().seed).expect("cache");
                let mut dataset = if flags.contains(RandomXFlag::FLAG_FULL_MEM) {
                    Some(RandomXDataset::new(flags, cache.clone(), 0).expect("dataset"))
                } else {
                    None
                };
                let mut vm =
                    RandomXVM::new(flags, Some(cache.clone()), dataset.clone()).expect("vm");
                let mut job = job_rx.get().clone();
                let mut target = job.difficulty();
                let mut nonce = thread_nonce_start;
                let mut accepted = 0;
                let mut hashes = 0;
                let mut last_report = Instant::now();

                tracing::debug!(
                    "Thread {i} starting with target: {}, nonce range: {}-{}",
                    target,
                    thread_nonce_start,
                    thread_nonce_end
                );
                loop {
                    if let Some(new_job) = job_rx.get_if_new() {
                        if new_job.seed != job.seed {
                            cache = RandomXCache::new(flags, &new_job.seed).expect("cache");
                            vm.reinit_cache(cache.clone()).ok();
                            if flags.contains(RandomXFlag::FLAG_FULL_MEM) {
                                dataset = Some(
                                    RandomXDataset::new(flags, cache.clone(), 0).expect("dataset"),
                                );
                                if let Some(ref d) = dataset {
                                    vm.reinit_dataset(d.clone()).ok();
                                }
                            }
                            tracing::debug!("Thread {i} reinitialized context with new job seed");
                        }
                        job = new_job;
                        target = job.difficulty();
                        nonce = thread_nonce_start;
                        accepted = 0;
                        last_report = Instant::now();
                    }
                    // Inside the worker thread loop:
                    if nonce <= thread_nonce_end {
                        hashes += 1;
                        if let Some(share) = job.next_share(&vm, nonce, target) {
                            accepted += 1;
                            tracing::debug!("Found share at nonce: {}", nonce);
                            if share_tx.send(share).is_err() {
                                break;
                            }
                        }
                        nonce += 1;
                    } else {
                        nonce = thread_nonce_start;
                    }
                    if last_report.elapsed().as_secs() >= 10 {
                        let hashrate = hashes as f64 / 10.0;
                        hashrate_tx.send(hashrate).unwrap_or(());
                        tracing::info!(
                            "Thread {i} - Hashrate: {:.2} H/s, Shares: {}",
                            hashrate,
                            accepted
                        );
                        hashes = 0;
                        last_report = Instant::now();
                    }
                }
            });
        }
        // Spawn hashrate aggregator thread
        thread::spawn(move || {
            let mut last_report = Instant::now();
            let mut total_hashrate = 0.0;
            let mut count = 0;
            loop {
                if let Ok(hashrate) = hashrate_rx.recv_timeout(std::time::Duration::from_secs(10)) {
                    total_hashrate += hashrate;
                    count += 1;
                    if count >= num_threads.get() || last_report.elapsed().as_secs() >= 10 {
                        tracing::info!("Total Hashrate: {:.2} H/s", total_hashrate);
                        total_hashrate = 0.0;
                        count = 0;
                        last_report = Instant::now();
                    }
                }
            }
        });
        Self { share_rx, job_tx }
    }
    pub fn try_recv_share(&self) -> Option<Share> {
        self.share_rx.try_recv().ok()
    }
    pub fn update_job(&self, job: Job) {
        let _ = self.job_tx.send(job);
    }
}
