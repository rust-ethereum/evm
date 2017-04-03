# SputnikVM: Ethereum Classic Virtual Machine

[![Build Status](https://travis-ci.org/ethereumproject/sputnikvm.svg?branch=master)](https://travis-ci.org/ethereumproject/sputnikvm)

## Project Description

This is separate implementation of Ethereum Virtual Machine that can
be integrated into Geth, Parity or other clients or may be be launched
as a standalone app for debugging purposes.

## Dependencies

Ensure you have at least `rustc 1.16.0 (30cf806ef 2017-03-10)`. Rust
1.15.0 and before is not supported.

## Stability Status:

- [x] Raw
- [ ] Draft
- [ ] Stable
- [ ] Deprecated
- [ ] Legacy

## Build Instructions

SputnikVM is written Rust. If you are not familiar with Rust please
see the
[getting started guide](https://doc.rust-lang.org/book/getting-started.html).

```
$ git clone git@github.com:ethereumproject/sputnikvm.git
$ cd sputnikvm
$ cargo build
$ RUST_LOG=svm cargo run --bin svm -- -g 23 -c 00 -d data
$ cargo run -- -h
```
