# Application User

## Step to get a basic solidity contract executed using SputnikVM

* ensure you have `solc` installed.
On `nixos` you type `nix-env -i solc` to install it.
* write a simple contract or use the accompanying `SimpleStorage.sol` contract.
```
pragma solidity ^0.4.0;

contract SimpleStorage {
    uint storedData;

    function set(uint x) {
        storedData = x;
    }

    function get() constant returns (uint) {
        return storedData;
    }
}
```
* execute `solc --bin -o SimpleStorage SimpleStorage.sol`
* then run these commands:
```
cd SimpleStorage
../../../target/debug/gaslighter cli -c SimpleStorage.bin
```

Voilà, that was your first hello world using SputnikVM.
