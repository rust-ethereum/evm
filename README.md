# Rust EVM

[![Build Status](https://github.com/rust-ethereum/evm/workflows/Rust/badge.svg)](https://github.com/rust-ethereum/evm/actions?query=workflow%3ARust)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE)

Rust EVM, also known as SputnikVM, is a flexible Ethereum Virtual Machine
interpreter that can be easily customized.

## Status

The Rust EVM project has a long history dating back to the initial
implementation in 2017 (when it was called SputnikVM). It has gone through
multiple rewrites over the years to accommodate for different requirements,
when we successfully tested one integrating Geth to sync the mainnet.

The current rewrite is used in production for the Frontier project (the
Ethereum-compatibility layer for Polkadot). However, we have not yet fully
tested it against Ethereum mainnet. If you have such requirements, PR for fixes
are welcomed.

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
