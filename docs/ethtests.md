# Debugging against the Ethereum test suite

When encountering a failed Ethereum test in the test suite repo, the following
method can be used for debugging.

Run the `jsontests` tool with `--debug` and `--write-failed` flag. This will
point out the entire trace and write the last failed test into the file
specified. For example:

```bash
cd jsontests
cargo run -- --debug --write-failed failed.json res/ethtests/GeneralStateTests/stEIP1559/
```

The `failed.json` file can then be compared with other implementations. For example, Geth:

```bash
go-ethereum/build/bin/evm --json --dump statetest failed.json
```
