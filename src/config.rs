//! Application configuration.

use std::net::SocketAddr;
use std::str::FromStr;

use anyhow::anyhow;
use clap::Parser;

/// Application configuration entry-point.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Storage implementation.
    #[arg(short = 's', long = "storage", env = "STORAGE", default_value_t = StorageConfig::InMemory)]
    pub storage: StorageConfig,

    /// JSON-RPC binding address.
    #[arg(short = 'a', long = "address", env = "ADDRESS", default_value = "0.0.0.0:3000")]
    pub address: SocketAddr,

    /// Number of EVM instances to run.
    #[arg(long = "evms", env = "EVMS", default_value = "1")]
    pub num_evms: usize,

    /// Number of threads to execute global async tasks.
    #[arg(long = "async-threads", env = "ASYNC_THREADS", default_value = "1")]
    pub num_async_threads: usize,

    /// Number of threads to execute global blocking tasks.
    #[arg(long = "blocking-threads", env = "BLOCKING_THREADS", default_value = "1")]
    pub num_blocking_threads: usize,
}

/// Storage configuration.
#[derive(Clone, Debug, strum::Display)]
pub enum StorageConfig {
    #[strum(serialize = "inmemory")]
    InMemory,

    #[strum(serialize = "postgres")]
    Postgres { url: String },
}

impl FromStr for StorageConfig {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        match s {
            "inmemory" => Ok(Self::InMemory),
            s if s.starts_with("postgres://") => Ok(Self::Postgres { url: s.to_string() }),
            s => Err(anyhow!("unknown storage: {}", s)),
        }
    }
}
