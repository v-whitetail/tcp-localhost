use clap::Parser;
use std::path::PathBuf;

#[derive(Debug,Parser)]
pub struct Cli {
    /// local ip address (typically 127.0.0.1)
    pub ip: String,
    /// localhost port to listen to (typically 8000 or 8080)
    pub port: u16,
    /// path to listen to (./ indicates the current folder)
    pub path: PathBuf,
}
impl Cli{
    pub fn args() -> Self {
        Cli::parse()
    }
}

