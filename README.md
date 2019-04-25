# SputnikVM: Rust Ethereum Virtual Machine Implementation

[![Build Status](https://travis-ci.org/ethereumproject/evm-rs.svg?branch=master)](https://travis-ci.org/ethereumproject/evm-rs)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE)

| Name               | Description                                   | Crates.io                                                                                                           | Documentation                                                                                        |
|--------------------|:---------------------------------------------:|:-------------------------------------------------------------------------------------------------------------------:|:----------------------------------------------------------------------------------------------------:|
| evm                | Core library for the Ethereum Virtual Machine | [![crates.io](https://img.shields.io/crates/v/evm.svg)](https://crates.io/crates/evm)                   | [![Documentation](https://docs.rs/evm/badge.svg)](https://docs.rs/evm)                   |
| evm-stateful       | Merkle Trie stateful wrapper for SputnikVM    | [![crates.io](https://img.shields.io/crates/v/evm-stateful.svg)](https://crates.io/crates/evm-stateful) | [![Documentation](https://docs.rs/evm-stateful/badge.svg)](https://docs.rs/evm-stateful) |

## Features

* **Partially verified (WIP)** - use various verification techniques to
  partially verify the correctness of functions.
* **Nightly** - take advantage of Rust nightly features, such as
  compiler plugins
* **Standalone** - can be launched as an independent process or integrated into other apps
* **Universal** - supports different Ethereum chains, such as ETC, ETH or private ones
* **Stateless** - only an execution environment connected to independent State storage
* **Fast** - main focus is on performance
* **IoT compatible** - designed to support hardware used in embedded devices
* written in Rust, can be used as a binary, cargo crate or shared
  library

## Supported Networks

* Foundation ([evm-network-foundation](./network/foundation))
* Classic ([evm-network-classic](./network/classic))
* Ellaism ([evm-network-ellaism](./network/ellaism))
* Expanse ([evm-network-expanse](./network/expanse))
* Musicoin ([evm-network-musicoin](./network/musicoin))
* Ubiq ([evm-network-ubiq](./network/ubiq))
* All of the above and other networks ([evm-network-dynamic](./network/dynamic))

## Supported Networks

| Network          | Crates.io                                                                                                                               | Documentation                                                                                                            |
|------------------|:---------------------------------------------------------------------------------------------------------------------------------------:|:------------------------------------------------------------------------------------------------------------------------:|
| Any Network      | [![crates.io](https://img.shields.io/crates/v/evm-network.svg)](https://crates.io/crates/evm-network)                       | [![Documentation](https://docs.rs/evm-network/badge.svg)](https://docs.rs/evm-network)                       |
| Ethereum Classic | [![crates.io](https://img.shields.io/crates/v/evm-network-classic.svg)](https://crates.io/crates/evm-network-classic)       | [![Documentation](https://docs.rs/evm-network-classic/badge.svg)](https://docs.rs/evm-network-classic)       |
| Ethereum         | [![crates.io](https://img.shields.io/crates/v/evm-network-foundation.svg)](https://crates.io/crates/evm-network-foundation) | [![Documentation](https://docs.rs/evm-network-foundation/badge.svg)](https://docs.rs/evm-network-foundation) |
| Ellaism          | [![crates.io](https://img.shields.io/crates/v/evm-network-ellaism.svg)](https://crates.io/crates/evm-network-ellaism)       | [![Documentation](https://docs.rs/evm-network-ellaism/badge.svg)](https://docs.rs/evm-network-ellaism)       |
| Ubiq             | [![crates.io](https://img.shields.io/crates/v/evm-network-ubiq.svg)](https://crates.io/crates/evm-network-ubiq)             | [![Documentation](https://docs.rs/evm-network-ubiq/badge.svg)](https://docs.rs/evm-network-ubiq)             |
| Expanse          | [![crates.io](https://img.shields.io/crates/v/evm-network-expanse.svg)](https://crates.io/crates/evm-network-expanse)       | [![Documentation](https://docs.rs/evm-network-expanse/badge.svg)](https://docs.rs/evm-network-expanse)       |
| Musicoin         | [![crates.io](https://img.shields.io/crates/v/evm-network-musicoin.svg)](https://crates.io/crates/evm-network-musicoin)     | [![Documentation](https://docs.rs/evm-network-musicoin/badge.svg)](https://docs.rs/evm-network-musicoin)     |

## Precompiled Contracts

The core library has the initial four precompiled contracts embedded. To use the bn128 and modexp precompiled contracts introduced by the Byzantium hard fork, pull the following crates.

| Name                         | Description                  | Crates.io                                                                                                                               | Documentation                                                                                                            |
|------------------------------|:----------------------------:|:---------------------------------------------------------------------------------------------------------------------------------------:|:------------------------------------------------------------------------------------------------------------------------:|
| evm-precompiled-bn128  | bn128 precompiled contracts  | [![crates.io](https://img.shields.io/crates/v/evm-precompiled-bn128.svg)](https://crates.io/crates/evm-precompiled-bn128)   | [![Documentation](https://docs.rs/evm-precompiled-bn128/badge.svg)](https://docs.rs/evm-precompiled-bn128)   |
| evm-precompiled-modexp | modexp precompiled contracts | [![crates.io](https://img.shields.io/crates/v/evm-precompiled-modexp.svg)](https://crates.io/crates/evm-precompiled-modexp) | [![Documentation](https://docs.rs/evm-precompiled-modexp/badge.svg)](https://docs.rs/evm-precompiled-modexp) |

## Related projects

* [ethereum-rs](https://github.com/etclabscore/ethereum-rs) -
  common traits and structs for Ethereum. 
* [etclient](https://source.that.world/source/etclient) -
  bare-minimal Ethereum client written in Rust.
* [SputnikVM FFI](https://github.com/ethereumproject/evm-ffi) - SputnikVM C and Go FFI bindings
* [SputnikVM Dev](https://github.com/ethereumproject/evm-dev) - SputnikVM instance for Smart Contract development, 
   provides testing environment and mock for JSON RPC API
* [SputnikVM in Browser](https://github.com/sorpaas/sputnikvm-in-browser) - experimental version of SputnikVM 
   compiled into WebAssembly, therefore can be launched in a browser on Node.js
* [SputnikVM for embedded devices](https://github.com/sorpaas/sputnikvm-on-rux) - experimental project to run on 
   full functional EVM on embedded devices       

## Dependencies

Ensure you have at least `rustc 1.33.0 (2aa4c46cf 2019-02-28)`. Rust 1.32.0 and
before is not supported.

## Documentation

* [Latest release documentation](https://docs.rs/evm)

## Build from sources

SputnikVM is written Rust. If you are not familiar with Rust please
see the
[getting started guide](https://doc.rust-lang.org/book/getting-started.html). 

### Build 

To start working with SputnikVM you'll 
need to install [rustup](https://www.rustup.rs/), then you can do:
 
```bash
$ git clone git@github.com:ethereumproject/evm-rs.git
$ cd evm-rs
$ cargo build --release --all
```

### Testing

We currently use two ways to test SputnikVM and ensure its execution
aligns with other Ethereum Virtual Machine implementations:

* [jsontests](/jsontests): This uses part of the Ethereum
  [tests](https://github.com/etclabscore/tests). Those tests
  currently do not have good coverage for system operation
  opcodes. Besides, some tests are incorrect so they are disabled.
* [regtests](/regtests): A complete regression tests is done on the
  Ethereum Classic mainnet from genesis block to block 4 million. Some
  of the previously failed tests are also integrated into Rust's test
  system.
  
 To run all tests, execute the following command in the cloned repository:
 ```bash
 $ cargo test --all
 ```
 
### Contribution

Formatting policies are described in [GUIDE.md](./GUIDE.md),
and the recommended automated formatting techniques may be found at [FORMATTING.md](./FORMATTING.md)

## License

Apache 2.0
