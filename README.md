# SputnikVM: A Blockchain Virtual Machine

[![Build Status](https://travis-ci.org/ethereumproject/sputnikvm.svg?branch=master)](https://travis-ci.org/ethereumproject/sputnikvm)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE)
[![Cargo](https://img.shields.io/crates/v/sputnikvm.svg)](https://crates.io/crates/sputnikvm)

SputnikVM is an implementation of an Ethereum Virtual Machine. It aims to be an
efficient, pluggable virtual machine for different Ethereum-based blockchains.

We encourage all Ethereum'esque blockchains to adopt SputnikVM, and to make use
of SputnikVM's [RFC governance project](https://etcrfc.that.world/) which
governs the parameters of each blockchain's VM. This way we can draw from the
experience of the community and learn from other proposed RFCs.

## Features

 * *Standalone* - can be launched as an independent process or integrated into other apps
 * *Universal* - supports different Ethereum chains, such as ETC, ETH or private ones
 * *Stateless* - only an execution environment connected to independent State storage
 * *Fast* - main focus is on performance
 * *IoT compatible* - designed to support hardware used in embedded devices
 * FFI, Protobuf and JSON interface
 * written in Rust, can be used as a binary, cargo crate or shared library  

## Related projects

 * [SputnikVM Dev](https://github.com/ethereumproject/sputnikvm-dev) - SputnikVM instance for Smart Contract development, 
    provides testing environment and mock for JSON RPC API
 * [SputnikVM in Browser](https://github.com/ethereumproject/sputnikvm-in-browser) - experimental version of SputnikVM 
    compiled into WebAssembly, therefore can be launched in a browser on Node.js
 * [SputnikVM for embedded devices](https://github.com/ethereumproject/sputnikvm-on-rux) - experimental project to run on 
    full functional EVM on embedded devices       

## Dependencies

Ensure you have at least `rustc 1.16.0 (30cf806ef 2017-03-10)`. Rust 1.15.0 and
before is not supported.

## Documentation

* [Latest release documentation](https://docs.rs/sputnikvm)
* [Unstable documentation](https://that.world/~docs/sputnikvm/sputnikvm)

## Build from sources

SputnikVM is written Rust. If you are not familiar with Rust please
see the
[getting started guide](https://doc.rust-lang.org/book/getting-started.html). 

### Build 

To start working with SputnikVM you'll 
need to install [rustup](https://www.rustup.rs/), then you can do:
 
```bash
$ git clone git@github.com:ethereumproject/sputnikvm.git
$ cd sputnikvm
$ cargo build --release --all
```

### Testing

We currently use two ways to test SputnikVM and ensure its execution
aligns with other Ethereum Virtual Machine implementations:

* [jsontests](/jsontests): This uses part of the Ethereum
  [tests](https://github.com/ethereumproject/tests). Those tests
  currently does not have good coverage for system operation
  opcodes. Besides, some tests are incorrect so they are disabled.
* [regtests](/regtests): A complete regression tests is done on the
  Ethereum Classic mainnet from genesis block to block 4 million. Some
  of the previously failed tests are also integrated into Rust's test
  system. See
  [wiki](https://github.com/ethereumproject/sputnikvm/wiki/Building-and-Testing)
  for how to reproduce the regression tests.
  
To learn more about building SputnikVM from source please read wiki page
 [Building and Testing](https://github.com/ethereumproject/sputnikvm/wiki/Building-and-Testing)  

## License

Apache 2.0