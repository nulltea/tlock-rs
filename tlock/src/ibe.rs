use std::ops::Mul;
use bls12_381_plus::{ExpandMsgXmd, G1Affine, G1Projective, G2Affine, G2Projective, Gt, Scalar};
use rand::distributions::Uniform;
use rand::{Rng, thread_rng};
use sha2::{Digest, Sha256};
use group::{Curve, GroupEncoding};
use itertools::Itertools;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ciphertext {
    pub u: G1Affine,
    pub v: Vec<u8>,
    pub w: Vec<u8>,
}

const BLOCK_SIZE: usize = 32;
pub const H2C_DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

pub fn encrypt<I: AsRef<[u8]>, M: AsRef<[u8]>>(master: G1Affine, id: I, msg: M) -> Ciphertext {
    assert!(msg.as_ref().len() <= BLOCK_SIZE, "plaintext too long for the block size");

    let mut rng = rand::thread_rng();
    // 1. Compute Gid = e(master,Q_id)
    let gid = {
        let qid = G2Projective::hash::<ExpandMsgXmd<Sha256>>(id.as_ref(), H2C_DST)
            .to_affine();

        bls12_381_plus::pairing(&master, &qid)
    };

    /// dirty fix: loop to sample randomness that won't mess up constant time operation.
    /// otherwise can `Scalar::from_bytes(r).unwrap()` panic from subtle crate
    let (sigma, r) = loop {
        // 2. Derive random sigma
        let sigma: [u8; BLOCK_SIZE] = (0..BLOCK_SIZE).map(|_| rng.sample(&Uniform::new(0u8, 8u8))).collect_vec().try_into().unwrap();

        // 3. Derive r from sigma and msg
        let r = {
            let mut hash = Sha256::new();
            hash.update(b"h3");
            hash.update(&sigma[..]);
            hash.update(msg.as_ref());
            let r = &hash.finalize().to_vec()[0..32].try_into().unwrap();

            Scalar::from_bytes(r)
        };

        if r.is_some().unwrap_u8() == 1u8 {
            break (sigma, r.unwrap());
        }
    };

    // 4. Compute U = G^r
    let g = G1Affine::generator();
    let u = (G1Affine::generator().mul(r)).to_affine();

    // 5. Compute V = sigma XOR H(rGid)
    let v = {
        let mut hash = sha2::Sha256::new();
        let r_gid = gid.mul(r);
        hash.update(b"h2"); // dst
        hash.update(&r_gid.to_bytes());
        let h_r_git = &hash.finalize().to_vec()[0..BLOCK_SIZE];

        xor(&sigma, h_r_git)
    };

    // 6. Compute W = M XOR H(sigma)
    let w = {
        let mut hash = sha2::Sha256::new();
        hash.update(b"h4");
        hash.update(&sigma[..]);
        let h_sigma = &hash.finalize().to_vec()[0..BLOCK_SIZE];
        xor(msg.as_ref(), h_sigma)
    };

    Ciphertext {
        u,
        v,
        w,
    }
}

pub fn decrypt(private: G2Affine, c: &Ciphertext) -> Vec<u8> {
    assert!(c.w.len() <= BLOCK_SIZE, "ciphertext too long for the block size");

    // 1. Compute sigma = V XOR H2(e(rP,private))
    let sigma = {
        let mut hash = sha2::Sha256::new();
        let r_gid = bls12_381_plus::pairing(&c.u, &private);
        hash.update(b"h2");
        hash.update(&r_gid.to_bytes());
        let h_r_git = &hash.finalize().to_vec()[0..BLOCK_SIZE];
        xor(h_r_git, &c.v)
    };

    // 2. Compute Msg = W XOR H4(sigma)
    let msg = {
        let mut hash = sha2::Sha256::new();
        hash.update(b"h4");
        hash.update(&sigma);
        let h_sigma = &hash.finalize().to_vec()[0..BLOCK_SIZE];
        xor(h_sigma, &c.w)
    };

    // 3. Check U = G^r
    let r_g = {
        let mut hash = sha2::Sha256::new();
        hash.update(b"h3");
        hash.update(&sigma[..]);
        hash.update(&msg);
        let r = &hash.finalize().to_vec()[0..BLOCK_SIZE];
        let r = Scalar::from_bytes(r.try_into().unwrap()).unwrap();
        (G1Affine::generator() * r).to_affine()
    };
    assert_eq!(c.u, r_g);

    msg
}

fn xor(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(a, b)| a ^ b).collect()
}
