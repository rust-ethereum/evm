# SputnikVM: Rust Ethereum Virtual Machine Implementation

[![Build Status](https://github.com/rust-blockchain/evm/workflows/Rust/badge.svg)](https://github.com/rust-blockchain/evm/actions?query=workflow%3ARust)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE)

| Name          | Description                                                     | Crates.io                                                                                                 | Documentation                                                                              |
|---------------|:---------------------------------------------------------------:|:---------------------------------------------------------------------------------------------------------:|:------------------------------------------------------------------------------------------:|
| evm           | Main library that re-exports most things.                        | [![crates.io](https://img.shields.io/crates/v/evm.svg)](https://crates.io/crates/evm)                     | [![Documentation](https://docs.rs/evm/badge.svg)](https://docs.rs/evm)                     |
| evm-core      | Core library defining the basic execution rules.                | [![crates.io](https://img.shields.io/crates/v/evm-core.svg)](https://crates.io/crates/evm-core)           | [![Documentation](https://docs.rs/evm-core/badge.svg)](https://docs.rs/evm-core)           |
| evm-gasometer | Integration of Ethereum gas rules.                              | [![crates.io](https://img.shields.io/crates/v/evm-gasometer.svg)](https://crates.io/crates/evm-gasometer) | [![Documentation](https://docs.rs/evm-gasometer/badge.svg)](https://docs.rs/evm-gasometer) |
| evm-runtime   | Runtime defining interface for block, transaction, and storage. | [![crates.io](https://img.shields.io/crates/v/evm-runtime.svg)](https://crates.io/crates/evm-runtime)     | [![Documentation](https://docs.rs/evm-runtime/badge.svg)](https://docs.rs/evm-runtime)     |

## Features

* **Standalone** - can be launched as an independent process or integrated into other apps
* **Universal** - supports different Ethereum chains, such as ETC, ETH or private ones
* **Stateless** - only an execution environment connected to independent State storage
* **Fast** - main focus is on performance
* written in Rust, can be used as a binary, cargo crate or shared
  library

## Dependencies

Ensure you have at least `rustc 1.51.0 (2fd73fabe 2021-03-23)`. Rust 1.50.0 and
before are not supported.

## Documentation

* [Latest release documentation](https://docs.rs/evm)

## Build from sources

SputnikVM is written in Rust. If you are not familiar with Rust, please
see the [starting documentation](https://www.rust-lang.org/learn).

### Build

To start working with SputnikVM you'll
need to install [rustup](https://www.rustup.rs/), then you can do:

```bash
$ git clone git@github.com:rust-blockchain/evm.git
$ cd evm
$ cargo build --release --all
```

## License

Apache 2.0
