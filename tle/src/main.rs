use std::process;
use anyhow::anyhow;
use std::fs;
use cli_batteries::version;
use tracing::{info, info_span, Instrument};
use tlock::client::Network;
use tlock::time;
use crate::args::{Options, Command, LockArgs, UnlockArgs};

mod args;

fn main() {
    cli_batteries::run(version!(), app);
}

async fn app(opts: Options) -> eyre::Result<()> {
    if let Some(command) = opts.command {
        match command {
            Command::Lock(args) => lock(args).await,
            Command::Unlock(args) => unlock(args).await,
        }.map_err(|e| eyre::anyhow!(e))?
    }

    Ok(())
}

async fn lock(args: LockArgs) -> anyhow::Result<()> {
    let network = Network::new(args.network_host, args.chain_hash).unwrap();
    let info = network.info().instrument(info_span!("getting network info")).await.unwrap();

    let round_number = match args.round_number {
        None => {
            let d = args.duration.expect("duration is expected if round_number isn't specified").into();
            time::round_after(info, d)
        },
        Some(n) => n,
    };

    info!("locked until {round_number} round");

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
