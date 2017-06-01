# VM Developer

## Building SputnikVM
```
$ git clone git@github.com:ethereumproject/sputnikvm.git
$ cd sputnikvm
$ cargo build --all
```
## SputnikVM Regression test
Steps to reproduce:
* Install Ethereum Classic Geth: `$ go install github.com/ethereumproject/go-ethereum/cmd/geth`.
* Run Geth with this command: `$ ~/go/bin/geth --rpc --rpcaddr 127.0.0.1 --rpcport 8545`.
* Wait for the chain to sync.
* Clone SputnikVM: `$ git clone git@github.com:ethereumproject/sputnikvm.git`
* Change directory into the `regtests` directory `$ cd regtests`
* Run this command: `$ RUST_BACKTRACE=1 cargo run --bin regtests -- -r http://127.0.0.1:8545`

## SputnikVM Common Unit Tests for all Ethereum implementations.
These tests are a clone of github.com/ethereumproject/tests
```
$ git clone git@github.com:ethereumproject/sputnikvm.git
$ cd sputnikvm/jsontests
$ cargo test
```
