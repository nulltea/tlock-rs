use std::time::Duration;
use anyhow::anyhow;
use bls12_381_plus::G1Affine;
use url::Url;
use serde::{Serialize, Deserialize};

pub struct Network {
    client: surf::Client,
    chain_hash: String
}

#[derive(Clone, Debug, Deserialize)]
struct ChainInfoResp {
    #[serde(with = "hex::serde")]
    pub public_key: Vec<u8>,
    pub hash: String,
    pub period: u64,
    pub genesis_time: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChainInfo {
    pub public_key: G1Affine,
    pub hash: String,
    pub period: Duration,
    pub genesis_time: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Beacon {
    pub round: u64,
    #[serde(with = "hex::serde")]
    pub randomness: Vec<u8>,
    #[serde(with = "hex::serde")]
    pub signature: Vec<u8>,
}

impl Network {
    pub fn new<S: AsRef<str>>(host: S, chain_hash: impl AsRef<str>) -> anyhow::Result<Self> {
        let url = {
            let base = Url::parse(host.as_ref())
                .map_err(|e| anyhow!("error parsing network host: {e}"))?;
            base.join(&format!("{}/", chain_hash.as_ref())).map_err(|e| anyhow!("error joining chain hash: {e}"))?
        };
        let config = surf::Config::new().set_base_url(url).set_timeout(None);
        Ok(Self {
            client: config.try_into()?,
            chain_hash: chain_hash.as_ref().to_string()
        })
    }

    pub async fn info(&self) -> anyhow::Result<ChainInfo> {
        let mut resp = self
            .client
            .get("info")
            .await
            .map_err(|e| anyhow!("error requesting info: {e}"))?;

        if resp.status() != 200 {
            return Err(anyhow!("{:?}", resp.body_string().await.unwrap()));
        }

        let res = resp
            .body_json::<ChainInfoResp>()
            .await
            .map_err(|e| anyhow!("error decoding info response: {e}"))?;

        let public_key = {
            let bytes = (&*res.public_key).try_into()
                .map_err(|e| anyhow!("invalid public key size"))?;
            G1Affine::from_compressed(bytes).unwrap()
        };

        Ok(ChainInfo{
            public_key,
            hash: res.hash,
            period: Duration::from_secs(res.period),
            genesis_time: res.genesis_time
        })
    }

    pub async fn get(&self, round: u64) -> anyhow::Result<Beacon> {
        let uri = if round == 0 {
            "public/latest".to_string()
        } else {
            format!("public/{round}")
        };

        let mut resp = self
            .client
            .get(uri)
            .await
            .map_err(|e| anyhow!("error requesting info: {e}"))?;

        if resp.status() != 200 {
            return Err(anyhow!("Too early"));
        }

        resp
            .body_json::<Beacon>()
            .await
            .map_err(|e| anyhow!("error decoding round response: {e}"))
    }
}
