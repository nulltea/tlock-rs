use std::time::Duration;
use duration_string::DurationString;
use gumdrop::{Opt, Options};

#[derive(Debug, Options, Clone)]
pub struct CLIArgs {
    help: bool,
    #[options(command)]
    pub command: Option<Command>,
}

#[derive(Debug, Options, Clone)]
pub enum Command {
    #[options(help = "Lock file until specified drand round")]
    Lock(LockArgs),
    #[options(help = "Try unlocking the file")]
    Unlock(UnlockArgs),
}

#[derive(Debug, Options, Clone)]
pub struct LockArgs {
    help: bool,

    #[options(free)]
    pub input_path: String,

    #[options(help = "write the result to the file at path OUTPUT", default = "./locked.pem")]
    pub output_path: String,

    #[options(help = "round number")]
    pub round_number: Option<u64>,

    #[options(help = "lock duration")]
    pub duration: Option<String>,

    #[options(help = "drand network host url", default = "https://pl-us.testnet.drand.sh")]
    pub network_host: String,

    #[options(help = "drand chain hash", default = "7672797f548f3f4748ac4bf3352fc6c6b6468c9ad40ad456a397545c6e2df5bf")]
    pub chain_hash: String,
}

#[derive(Debug, Options, Clone)]
pub struct UnlockArgs {
    help: bool,

    #[options(free)]
    pub input_path: String,

    #[options(help = "write the result to the file at path OUTPUT", default = "./unlocked.pem")]
    pub output_path: String,

    #[options(help = "drand network host url", default = "https://pl-us.testnet.drand.sh")]
    pub network_host: String,

    #[options(help = "drand chain hash", default = "7672797f548f3f4748ac4bf3352fc6c6b6468c9ad40ad456a397545c6e2df5bf")]
    pub chain_hash: String,
}
