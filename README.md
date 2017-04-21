# SputnikVM: Ethereum Classic Virtual Machine

[![Build Status](https://travis-ci.org/ethereumproject/sputnikvm.svg?branch=master)](https://travis-ci.org/ethereumproject/sputnikvm)

## Project Description

This is an implementation of Ethereum Virtual Machine that can be integrated into Geth, Parity or other clients or may be be launched as a standalone app for debugging purposes.

## Problem 1

As the Ethereum Classic Improvement Proposal process advances it'll become harder for the many and varied Ethereum Classic clients to keep up with these changes.

## Solution

Having the Virtual Machine in library form allows us as a community to cooperate and focus expertise on one VM. Clients that use SputnikVM will have a larger community of users that'll be able to navigate the problem domain at a much faster pace than an Ethereum Classic client that chooses to implement and maintain their own virtual machine.

## Problem 2

Unless a business is specifically setup to earn money from supporting code, it becomes hard for that business to use GPLv3 licensed code, as the GPLv3 requires them to GPLv3 sublicense any source code that links to a GPLv3 Ethereum Virtual Machine.

## Solution

SputnikVM is licensed under the Apache 2 License. Businesses may use the VM without relicensing their code. Copyright is spread throughout the community, meaning the code isn't easily susceptible to corporate hijacking and relicensing thereafter. It's requested that if you use this code you give proper acknowledgement and also we encourage you to send patches upstream. Please don't develop in the dark.

## Problem 3

Most Ethereum Virtual Machines are tied up and buried in an Ethereum Client implemenation. Forcing you to use their Virtual Machine.

## Solution

We've deliberately designed SputnikVM to be portable and accessible in different ways. This will encourage Ethereum Classic clients to use SputnikVM as their Virtual Machine. The methods of access are:

* Rust crate
* Foreign Function Interface
* Socket connection
* Command Line Interface

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
$ RUST_LOG=gaslighter cargo run --bin gaslighter -- -k ffi -s target/debug/libsputnikvm.so --capnp_test_bin tests.bin --run_test //
```
for a quick compile, test, code cycle, run this command chain:

```
capnp eval --binary tests/mod.capnp all > tests.bin && RUST_BACKTRACE=1 RUST_LOG=gaslighter cargo run --bin gaslighter -- -k ffi -s target/debug/libsputnikvm.so -t tests.bin -r ///
```
