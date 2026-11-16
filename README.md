# Rust EVM

[![Build Status](https://github.com/rust-ethereum/evm/workflows/Rust/badge.svg)](https://github.com/rust-ethereum/evm/actions?query=workflow%3ARust)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE)
[![Matrix](https://img.shields.io/matrix/evm%3Amatrix.org)](https://matrix.to/#/#evm:matrix.org)
[![Crates.io Version](https://img.shields.io/crates/v/evm)](https://crates.io/crates/evm)
[![docs.rs](https://img.shields.io/docsrs/evm)](https://docs.rs/evm)

Rust EVM, also known as SputnikVM, is a flexible Ethereum Virtual Machine
interpreter that can be easily customized.

## Status

The Rust EVM project has a long history dating back to the initial
implementation in 2017 (when it was called SputnikVM). It has gone through
multiple rewrites over the years to accommodate for different requirements,
when we successfully tested it by integrating Geth to sync the mainnet.

The current rewrite is used in production for the Frontier project (the
Ethereum-compatibility layer for Polkadot). We pass the Ethereum test suite. PRs
for fixes are welcomed.

## The 0.x line

The 0.x line, released as `evm` up to 0.43.4 and formerly the `v0.x` branch of
this repository, has moved to
[rust-ethereum/evm-legacy](https://github.com/rust-ethereum/evm-legacy), where
its fixes and releases happen. From 0.44.0 on, its main crate is published as
`evm-legacy`. Its library is still named `evm`, so a project on the 0.x line
changes only its dependency, to `evm-legacy = "0.44"`.

## Features

* **Standalone** - can be launched as an independent process or integrated into other apps.
* **Flexible** - can be customized and extended to support additional opcodes,
  additional precompiles, different gasometers or other more exotic use cases.
* **Portable** - support `no_std`, and can be used in different environments
  like in WebAssembly.
* **Fast** - we of course try to be fast!
* written in Rust, can be used as a binary, cargo crate or shared library.

## Dependencies

Rust EVM requires at least `rustc 1.75`.

## Documentation

* [Latest release documentation](https://docs.rs/evm)

## License

Apache 2.0
