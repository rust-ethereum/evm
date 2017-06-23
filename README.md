# SputnikVM: A Blockchain Virtual Machine

[![Build Status](https://travis-ci.org/ethereumproject/sputnikvm.svg?branch=master)](https://travis-ci.org/ethereumproject/sputnikvm)
[![Coverage Status](https://coveralls.io/repos/github/ethereumproject/sputnikvm/badge.svg?branch=master)](https://coveralls.io/github/ethereumproject/sputnikvm?branch=master)
[![Cargo](https://img.shields.io/crates/v/sputnikvm.svg)](https://crates.io/crates/sputnikvm)

* [Latest release documentation](https://docs.rs/sputnikvm)
* [Unstable documentation](https://that.world/~docs/sputnikvm/sputnikvm)

SputnikVM is an implementation of an Ethereum Virtual Machine. It aims to be an
efficient, pluggable virtual machine for different blockchains.

We encourage all Ethereum'esque blockchains to adopt SputnikVM, and to make use
of SputnikVM's [RFC governance project](https://etcrfc.that.world/) which
governs the parameters of each blockchain's VM. This way we can draw from the
experience of the community and learn from other proposed RFCs.

## Reasoning

__RFC Advancement__: As the Ethereum Classic Request For Comment (RFC)
process advances, it'll become harder for the many and varied Ethereum Classic
clients to keep up with these changes.

> Having the Virtual Machine in library form allows us as a community to
cooperate and focus expertise on one VM. Clients that use SputnikVM will have a larger
community of users that'll be able to navigate the problem domain at a much
faster pace than an Ethereum Classic client that chooses to implement and
maintain their own virtual machine.

__Licensing__: Unless a business is specifically setup to earn money from
supporting code, it becomes hard for that business to use _GPLv3_ licensed
code, as the _GPLv3_ requires them to _GPLv3_ sublicense any source code that
links to a _GPLv3_ Ethereum Virtual Machine.

> SputnikVM is licensed under the Apache 2 License. Businesses may use the VM
without relicensing their code. Copyright is spread throughout the community,
meaning the code isn't easily susceptible to corporate hijacking and
relicensing thereafter. It's requested that if you use this code, you give
proper acknowledgement, and we also encourage you to send patches upstream.
Please don't develop in the dark.

__Accessiblity__: Most Ethereum Virtual Machines are tied up and buried in an
Ethereum Client implemenation, forcing you to use their Virtual Machine.

> We've deliberately designed SputnikVM to be portable and accessible in different
ways. This will encourage Ethereum Classic clients to use SputnikVM as their
Virtual Machine. The methods of access are:
>
> * Rust crate
> * Foreign Function Interface
> * Socket connection
> * Command Line Interface

## Dependencies

Ensure you have at least `rustc 1.16.0 (30cf806ef 2017-03-10)`. Rust 1.15.0 and
before is not supported.

## Build Instructions

SputnikVM is written Rust. If you are not familiar with Rust please
see the
[getting started guide](https://doc.rust-lang.org/book/getting-started.html).

* [Building and Testing](https://github.com/ethereumproject/sputnikvm/wiki/Building-and-Testing)
