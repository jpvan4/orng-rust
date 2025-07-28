extern crate core;
pub mod error;
pub mod job;
pub mod share;
pub mod stratum;
pub mod worker;

// Re-export main types for easy access
pub use error::{Error, Result};
pub use job::Job;
pub use share::Share;
pub use stratum::Stratum;
pub use worker::Worker;
