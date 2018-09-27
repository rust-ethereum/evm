# SputnikVM: Rust Ethereum Virtual Machine Implementation

[![Build Status](https://travis-ci.org/etclabscore/sputnikvm.svg?branch=master)](https://travis-ci.org/etclabscore/sputnikvm)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE)

| Name               | Description                                   | Crates.io                                                                                                           | Documentation                                                                                        |
|--------------------|:---------------------------------------------:|:-------------------------------------------------------------------------------------------------------------------:|:----------------------------------------------------------------------------------------------------:|
| sputnikvm          | Core library for the Ethereum Virtual Machine | [![crates.io](https://img.shields.io/crates/v/sputnikvm.svg)](https://crates.io/crates/sputnikvm)                   | [![Documentation](https://docs.rs/sputnikvm/badge.svg)](https://docs.rs/sputnikvm)                   |
| sputnikvm-stateful | Merkle Trie stateful wrapper for SputnikVM    | [![crates.io](https://img.shields.io/crates/v/sputnikvm-stateful.svg)](https://crates.io/crates/sputnikvm-stateful) | [![Documentation](https://docs.rs/sputnikvm-stateful/badge.svg)](https://docs.rs/sputnikvm-stateful) |

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

* Foundation (evm-network-foundation)
* Classic (evm-network-classic)
* Ellaism (evm-network-ellaism)
* Expanse (evm-network-expanse)
* Musicoin (evm-network-musicoin)
* Ubiq (evm-network-ubiq)

## Supported Networks

| Network          | Crates.io                                                                                                                               | Documentation                                                                                                            |
|------------------|:---------------------------------------------------------------------------------------------------------------------------------------:|:------------------------------------------------------------------------------------------------------------------------:|
| Ethereum Classic | [![crates.io](https://img.shields.io/crates/v/sputnikvm-network-classic.svg)](https://crates.io/crates/sputnikvm-network-classic)       | [![Documentation](https://docs.rs/sputnikvm-network-classic/badge.svg)](https://docs.rs/sputnikvm-network-classic)       |
| Ethereum         | [![crates.io](https://img.shields.io/crates/v/sputnikvm-network-foundation.svg)](https://crates.io/crates/sputnikvm-network-foundation) | [![Documentation](https://docs.rs/sputnikvm-network-foundation/badge.svg)](https://docs.rs/sputnikvm-network-foundation) |
| Ellaism          | [![crates.io](https://img.shields.io/crates/v/sputnikvm-network-ellaism.svg)](https://crates.io/crates/sputnikvm-network-ellaism)       | [![Documentation](https://docs.rs/sputnikvm-network-ellaism/badge.svg)](https://docs.rs/sputnikvm-network-ellaism)       |
| Ubiq             | [![crates.io](https://img.shields.io/crates/v/sputnikvm-network-ubiq.svg)](https://crates.io/crates/sputnikvm-network-ubiq)             | [![Documentation](https://docs.rs/sputnikvm-network-ubiq/badge.svg)](https://docs.rs/sputnikvm-network-ubiq)             |
| Expanse          | [![crates.io](https://img.shields.io/crates/v/sputnikvm-network-expanse.svg)](https://crates.io/crates/sputnikvm-network-expanse)       | [![Documentation](https://docs.rs/sputnikvm-network-expanse/badge.svg)](https://docs.rs/sputnikvm-network-expanse)       |
| Musicoin         | [![crates.io](https://img.shields.io/crates/v/sputnikvm-network-musicoin.svg)](https://crates.io/crates/sputnikvm-network-musicoin)     | [![Documentation](https://docs.rs/sputnikvm-network-musicoin/badge.svg)](https://docs.rs/sputnikvm-network-musicoin)     |

## Precompiled Contracts

The core library has the initial four precompiled contracts embedded. To use the bn128 and modexp precompiled contracts introduced by the Byzantium hard fork, pull the following crates.

| Name                         | Description                  | Crates.io                                                                                                                               | Documentation                                                                                                            |
|------------------------------|:----------------------------:|:---------------------------------------------------------------------------------------------------------------------------------------:|:------------------------------------------------------------------------------------------------------------------------:|
| sputnikvm-precompiled-bn128  | bn128 precompiled contracts  | [![crates.io](https://img.shields.io/crates/v/sputnikvm-precompiled-bn128.svg)](https://crates.io/crates/sputnikvm-precompiled-bn128)   | [![Documentation](https://docs.rs/sputnikvm-precompiled-bn128/badge.svg)](https://docs.rs/sputnikvm-precompiled-bn128)   |
| sputnikvm-precompiled-modexp | modexp precompiled contracts | [![crates.io](https://img.shields.io/crates/v/sputnikvm-precompiled-modexp.svg)](https://crates.io/crates/sputnikvm-precompiled-modexp) | [![Documentation](https://docs.rs/sputnikvm-precompiled-modexp/badge.svg)](https://docs.rs/sputnikvm-precompiled-modexp) |

## Related projects

<<<<<<< HEAD
* [ethereum-rs](https://github.com/etclabscore/ethereum-rs) -
  common traits and structs for Ethereum. 
* [etclient](https://source.that.world/source/etclient) -
  bare-minimal Ethereum client written in Rust.
=======
 * [SputnikVM Dev](https://github.com/ETCDEVTeam/sputnikvm-dev) - SputnikVM instance for Smart Contract development, 
    provides testing environment and mock for JSON RPC API
 * [SputnikVM in Browser](https://github.com/sorpaas/sputnikvm-in-browser) - experimental version of SputnikVM 
    compiled into WebAssembly, therefore can be launched in a browser on Node.js
 * [SputnikVM for embedded devices](https://github.com/sorpaas/sputnikvm-on-rux) - experimental project to run on 
    full functional EVM on embedded devices       
>>>>>>> Fixed all github links, Fixed the travis badge

## Dependencies

Ensure you have at least `rustc 1.26.2 (594fb253c 2018-06-01)`. Rust 1.25.0 and
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
$ git clone git@github.com:etclabscore/sputnikvm.git
$ cd sputnikvm
$ cargo build --release --all
```

### Testing

We currently use two ways to test SputnikVM and ensure its execution
aligns with other Ethereum Virtual Machine implementations:

* [jsontests](/jsontests): This uses part of the Ethereum
  [tests](https://github.com/etclabscore/tests). Those tests
  currently does not have good coverage for system operation
  opcodes. Besides, some tests are incorrect so they are disabled.
* [regtests](/regtests): A complete regression tests is done on the
  Ethereum Classic mainnet from genesis block to block 4 million. Some
  of the previously failed tests are also integrated into Rust's test
  system. See
  [wiki](https://github.com/ETCDEVTeam/sputnikvm/wiki/Building-and-Testing)
  for how to reproduce the regression tests.
  
To learn more about building SputnikVM from source please read wiki page
 [Building and Testing](https://github.com/ETCDEVTeam/sputnikvm/wiki/Building-and-Testing)  

## License

Apache 2.0
