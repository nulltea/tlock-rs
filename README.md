# rs-tlock: Practical Timelock Encryption/Decryption in Rust

This repo contains pure Rust implementation of [drand/tlock](https://github.com/drand/tlock) scheme. It provides time-based encryption and decryption capabilities by relying on a [drand](https://drand.love/) threshold network and identity-based encryption (IBE). The IBE scheme implemented here is [*Boneh-Franklin*](https://crypto.stanford.edu/~dabo/papers/bfibe.pdf).

## Usage
The tlock system relies on an unchained drand network. Working endpoints to access it are, for now:
- https://pl-us.testnet.drand.sh/
- https://testnet0-api.drand.cloudflare.com/

### Lock file for given duration
```bash
cargo run -- lock -o test_lock.pem -d 30s test.txt
```

### Lock file for drand round
```bash
cargo run -- lock -o test_lock.pem -r 1000 test.txt
```

### Attempt unlocking file
```bash
cargo run -- unlock -o test_unlock.txt test_lock.pem
```

Error `Too early` will appear, if one tries to unlock a file before the specified round is reached.
