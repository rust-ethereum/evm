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
$ capnp eval --binary tests/mod.capnp all > tests.bin
$ RUST_LOG=gaslighter cargo run --bin gaslighter -- --capnp_test_bin tests.bin --run_test //
```
for a quick compile tests then code, run this command chain:

```
$ capnp eval --binary tests/mod.capnp all > tests.bin && RUST_LOG=gaslighter cargo run --bin gaslighter -- -t tests.bin -r //
```
