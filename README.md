# Muta SDK

The RUST SDK for [Muta](https://github.com/nervosnetwork/muta)(a High performance Blockchain framework). Allow you to interact with Muta node's GraphQL service.

## Quick Start

### cargo

```toml
[dependencies]
muta-sdk = "0.1"
```

## Usage

```rust
use muta_sdk;

let wallet = muta_sdk::wallet::Wallet::generate("");
let private_key = wallet.derive_privatekey(0);

let account = muta_sdk::account::Account::new(private_key);
let address = account.get_address();
println!("address {}", address);
```

## Modules

There are several mod you may interest on as follows.

- [Wallet](./src/wallet.rs) - BIP44 wallet
- [Account](./src/account.rs) - Account for managing private key and signing
- [Client](./src/client) - Wrapping the GraphQL like RPC
- [Util](./src/util.rs) - Several utilities

## Links

- [Tutorial](./docs/tutorial.md)
- [Examples](./examples)
- [Muta](https://github.com/nervosnetwork/muta)

## Development

> To ensure that the test passes, please enable a muta-example chain before the test.
> To start a muta-example chain, please reference [Muta Documentation](https://docs.muta.dev/docs).

```shell
git clone https://github.com/nervosnetwork/muta-sdk-rust
cd muta-sdk-rust
cargo build
cargo test
```
