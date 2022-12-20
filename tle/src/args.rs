use std::time::Duration;
use clap::{Args, Parser};
// use clap_duration::duration_range_value_parse;

#[derive(Clone, Parser)]
pub struct Options {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Clone, clap::Subcommand)]
pub enum Command {
    #[command(about = "Lock file until specified drand round")]
    Lock(LockArgs),

    #[command(about = "Try unlocking the file")]
    Unlock(UnlockArgs),
}

#[derive(Clone, Args)]
pub struct LockArgs {
    #[clap(index = 1, help = "plaintext file path")]
    pub input_path: String,

    #[clap(short, long, default_value = "./locked.pem", help = "write the result to the file at path OUTPUT")]
    pub output_path: String,

    #[clap(short, long, help = "round number")]
    pub round_number: Option<u64>,

    #[clap(short, long, help = "lock file for duration (y/w/d/h/m/s/ms)")]
    pub duration: Option<humantime::Duration>,

    #[clap(short, long, default_value = "https://pl-us.testnet.drand.sh", help = "drand network host url")]
    pub network_host: String,

    #[clap(short, long, default_value = "7672797f548f3f4748ac4bf3352fc6c6b6468c9ad40ad456a397545c6e2df5bf", help = "drand chain hash")]
    pub chain_hash: String,
}

#[derive(Clone, Args)]
pub struct UnlockArgs {
    #[clap(index = 1, help = "ciphertext file path")]
    pub input_path: String,

    #[clap(short, long, default_value = "./unlocked.pem", help = "write the result to the file at path OUTPUT")]
    pub output_path: String,

    #[clap(short, long, default_value = "https://pl-us.testnet.drand.sh", help = "drand network host url")]
    pub network_host: String,

    #[clap(short, long, default_value = "7672797f548f3f4748ac4bf3352fc6c6b6468c9ad40ad456a397545c6e2df5bf", help = "drand chain hash")]
    pub chain_hash: String,
}
