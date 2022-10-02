use std::ops::Add;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use crate::client::ChainInfo;

pub fn round_at(chain_info: ChainInfo, t: SystemTime) -> u64 {
    let since_epoch = t.duration_since(UNIX_EPOCH).unwrap();
    let t_unix = since_epoch.as_secs();
    current_round(t_unix, chain_info.period, chain_info.genesis_time)
}

pub fn round_after(chain_info: ChainInfo, d: Duration) -> u64 {
    let t = SystemTime::now().add(d);
    round_at(chain_info, t)
}

pub fn current_round(now: u64, period: Duration, genesis: u64) -> u64 {
    let (next_round, _) = next_round(now, period, genesis);

    if next_round <= 1 {
        next_round
    } else {
        next_round - 1
    }
}

pub fn next_round(now: u64, period: Duration, genesis: u64) -> (u64, u64) {
    if now < genesis {
        return (1, genesis)
    }

    let from_genesis = now - genesis;
    let next_round = (((from_genesis as f64)/ (period.as_secs() as f64)).floor() + 1f64) as u64;
    let next_time = genesis + next_round*period.as_secs();

    (next_round, next_time)
}
