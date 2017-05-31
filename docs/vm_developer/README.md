# VM Developer

* Building SputnikVM
```
$ git clone git@github.com:ethereumproject/sputnikvm.git
$ cd sputnikvm
$ cargo build
```
* SputnikVM Regression test

```
$ go install github.com/ethereumproject/go-ethereum/cmd/geth
$ ~/go/bin/geth --rpc --rpcaddr 127.0.0.1 --rpcport 8888
# wait for the blockchain to sync
$ <ctrl-c>
$ git clone git@github.com:ethereumproject/sputnikvm.git
$ cd sputnikvm/regtests
$ RUST_BACKTRACE=1 RUST_LOG=regression_test cargo run --bin regression_test -- -k reg -r localhost:8888
```
