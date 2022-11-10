pub mod client;
pub mod ibe;
pub mod time;

use std::io;
use anyhow::anyhow;
use bls12_381_plus::{G1Affine, G2Affine};
use sha2::Digest;
use crate::ibe::Ciphertext;
use crate::client::{Beacon, Network};

pub async fn encrypt<W: io::Write, R: io::Read>(network: Network, mut dst: W, mut src: R, round_number: u64) -> anyhow::Result<()> {
    let info = network.info().await?;

    let mut message = [0; 32];
    src.read(&mut message).map_err(|e| anyhow!("error reading {e}"))?;

    let ct = time_lock(info.public_key, round_number, message);

    {
        let mut buffer = unsigned_varint::encode::u64_buffer();
        dst.write_all(unsigned_varint::encode::u64(round_number, &mut buffer)).unwrap();
    }

    dst.write_all(ct.u.to_compressed().as_ref()).unwrap();
    dst.write_all(&ct.v).unwrap();
    dst.write_all(&ct.w).unwrap();

    Ok(())
}

pub async fn decrypt<W: io::Write, R: io::Read>(network: Network, mut dst: W, mut src: R) -> anyhow::Result<()> {
    let round = unsigned_varint::io::read_u64(&mut src).map_err(|e| anyhow!("error reading {e}"))?;

    let c = {
        let mut u = [0u8;48];
        src.read_exact(&mut u).map_err(|e| anyhow!("error reading {e}"))?;
        let mut v = [0u8;32];
        src.read_exact(&mut v).map_err(|e| anyhow!("error reading {e}"))?;
        let mut w = [0u8;32];
        src.read_exact(&mut w).map_err(|e| anyhow!("error reading {e}"))?;

        Ciphertext{
            u: G1Affine::from_compressed(&u).unwrap(),
            v: v.to_vec(),
            w: w.to_vec(),
        }
    };

    let beacon = network.get(round).await?;
    let mut pt = time_unlock(beacon, &c);

    if let Some(i) = pt.iter().rposition(|x| *x != 0) {
        pt.truncate(i+1);
    }

    dst.write_all(&pt).map_err(|e| anyhow!("error write {e}"))
}

pub fn time_lock<M: AsRef<[u8]>>(pub_key: G1Affine, round_number: u64, message: M) -> ibe::Ciphertext {
    let id = {
        let mut hash = sha2::Sha256::new();
        hash.update(&round_number.to_be_bytes());
        &hash.finalize().to_vec()[0..32]
    };

    ibe::encrypt(pub_key, id, message)
}

pub fn time_unlock(beacon: Beacon, c: &Ciphertext) -> Vec<u8> {
    let private = G2Affine::from_compressed((&*beacon.signature).try_into().unwrap()).unwrap();

    ibe::decrypt(private, c)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use crate::client::ChainInfo;
    use super::*;

    #[test]
    fn test_e2e() {
        let pk_bytes = hex::decode("8200fc249deb0148eb918d6e213980c5d01acd7fc251900d9260136da3b54836ce125172399ddc69c4e3e11429b62c11").unwrap();
        let info = ChainInfo {
            public_key: G1Affine::from_compressed((&*pk_bytes).try_into().unwrap()).unwrap(),
            hash: "7672797f548f3f4748ac4bf3352fc6c6b6468c9ad40ad456a397545c6e2df5bf".to_string(),
            period: Duration::new(5, 0),
            genesis_time: 0
        };

        let msg = vec![8;32];
        let ct = time_lock(info.public_key, 1000, msg.clone());

        let beacon = Beacon {
            round: 1000,
            randomness: hex::decode("3467f5d3118af125fbe8ffa0272e9fd1df026702afd4da50d0a0c8b3ff2dbf21").unwrap(),
            signature: hex::decode("a4721e6c3eafcd823f138cd29c6c82e8c5149101d0bb4bafddbac1c2d1fe3738895e4e21dd4b8b41bf007046440220910bb1cdb91f50a84a0d7f33ff2e8577aa62ac64b35a291a728a9db5ac91e06d1312b48a376138d77b4d6ad27c24221afe").unwrap(),
        };


        let pt = time_unlock(beacon, &ct);
        assert_eq!(pt, msg)
    }
}
