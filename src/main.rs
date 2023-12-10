#![allow(unused,dead_code)]

use anyhow::Result;
use tcp_localhost::{
    homepage::*,
    utils::*,
    cli::*,
};

#[tokio::main]
async fn main() -> Result<()>{

    let args = Cli::args();

    startup(&args.path);

    tokio::join!(
        host(&args.ip, &args.port, &args.path),
        watch_dir(&args.path),
        );

    Ok(())
}

