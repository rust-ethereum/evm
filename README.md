# evm-rs: Rust Ethereum Virtual Machine Implementation

evm-rs is an Ethereum Virtual Machine implementation, and is the
nightly version of
[SputnikVM](https://github.com/ethereumproject/sputnikvm). It is also
one of the implementation that supports the [ethoxy
specs](https://github.com/ethoxy/specs) initiative.

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

## Related projects

* [ethereum-rs](https://source.that.world/source/ethereum-rs) -
  common traits and structs for Ethereum. Nightly development branch
  of [etcommon-rs](https://github.com/ethereumproject/etcommon-rs)
  and support branch for evm-rs.
* [etclient](https://source.that.world/source/etclient) -
  bare-minimal Ethereum client written in Rust.

## Dependencies

Ensure you have at least `rustc 1.16.0 (30cf806ef 2017-03-10)`. Rust 1.15.0 and
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
 
```lang=bash
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
