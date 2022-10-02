use tlock::client::Network;

#[tokio::main]
async fn main() {
    let client = Network::new("https://pl-us.testnet.drand.sh/", "7672797f548f3f4748ac4bf3352fc6c6b6468c9ad40ad456a397545c6e2df5bf").unwrap();
    let info = client.info().await.unwrap();

    let msg = vec![8;32];
    let ct = tlock::time_lock(info.public_key, 1000, &msg);

    let beacon = client.get(1000).await.unwrap();

    let pt = tlock::time_unlock(beacon, &ct);

    assert_eq!(msg, pt);
}
