use std::process;
use anyhow::anyhow;
use std::fs;
use duration_string::DurationString;
use gumdrop::Options;
use tlock::client::Network;
use tlock::time;
use crate::args::{CLIArgs, Command, LockArgs, UnlockArgs};

mod args;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: CLIArgs = CLIArgs::parse_args_default_or_exit();
    let command = args.command.unwrap_or_else(|| {
        eprintln!("[command] is required");
        eprintln!("{}", CLIArgs::usage());
        process::exit(2)
    });

    match command {
        Command::Lock(args) => lock(args).await?,
        Command::Unlock(args) => unlock(args).await?,
    }

    Ok(())
}

async fn lock(args: LockArgs) -> anyhow::Result<()> {
    let network = Network::new(args.network_host, args.chain_hash).unwrap();
    let info = network.info().await.unwrap();

    let round_number = match args.round_number {
        None => {
            let d = DurationString::from_string(
                args.duration.expect("duration is expected if round_number isn't specified")
            ).map_err(|e| anyhow!("error parsing duration: {e}"))?.into();
            time::round_after(info, d)
        },
        Some(n) => n,
    };

    println!("locked until {round_number} round");

    let src = fs::File::open(args.input_path).map_err(|e| anyhow!("error reading input file"))?;
    let dst = fs::File::create(args.output_path).map_err(|e| anyhow!("error creating output file"))?;

    tlock::encrypt(network, dst, src, round_number).await
}


async fn unlock(args: UnlockArgs) -> anyhow::Result<()> {
    let network = Network::new(args.network_host, args.chain_hash).unwrap();

    let src = fs::File::open(args.input_path).map_err(|e| anyhow!("error reading input file"))?;
    let dst = fs::File::create(args.output_path).map_err(|e| anyhow!("error creating output file"))?;

    tlock::decrypt(network, dst, src).await
}
