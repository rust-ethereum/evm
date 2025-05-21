## Approach to fuzzing
This fuzzing harness uses a "structure-aware" approach by using [arbitrary](https://github.com/rust-fuzz/arbitrary) to fuzz the EVM interpreter.

The gas limit is set to ``400_000``, which is typical for complex evm transactions.

The gas price is set to 30, which is the average gas price on [moonbeam](https://moonscan.io/gastracker).

The harness has four invariants:
1. The user MUST spend funds to execute bytecode.
2. The user MUST not go above the gas_limit.
3. There should be NO funds minted.
4. The execution time MUST be within a reasonable thresold.

## Analysis
Additionally, the harness is instrumented with memory profiling to spot anomoalies.
When the inputs are run with ``cargo ziggy run``, the harness will print
execution time, and consumption of memory and storage. This can be analyzed
manually to spot any anomalies.

## Orchestrating the campaign
Fuzzing is orchestrated by [ziggy](https://github.com/srlabs/ziggy/).

It uses [AFL++](https://github.com/AFLplusplus/AFLplusplus/) and [honggfuzz](https://github.com/google/honggfuzz) under the hood.

Please refer to its documentation for details.

Quickstart command to fuzz:

``` bash
cargo ziggy fuzz -j$(nproc) -t1
```
