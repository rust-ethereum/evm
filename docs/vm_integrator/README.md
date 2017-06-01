# VM User

## Methods of accessing `SputnikVM`:
* Foreign Function interface
* Service level asynchronous IO
* Command Line Interface
* Via the standard `Cargo` build system.

## Require/Commit functionality of the VM.

SputnikVM for the moment has taken a rather functional approach to executing the VM.

A code snippet from the regression test demonstrates the setup and execution.

``` rust
let mut vm = SeqVM::new(context, block_header.clone(), &FRONTIER_PATCH);
loop {
    match vm.fire() {
        Ok(()) => {
            println!("VM exited successfully, checking results ...");
            break;
        },
        Err(RequireError::Account(address)) => {
            println!("Feeding VM account at 0x{:x} ...", address);
            let nonce = M256::from_str(&client.get_transaction_count(&format!("0x{:x}", address),
                                                                     &block.number)).unwrap();
            let balance = U256::from_str(&client.get_balance(&format!("0x{:x}", address),
                                                            &block.number)).unwrap();
            let code = read_hex(&client.get_code(&format!("0x{:x}", address),
                                                 &block.number)).unwrap();
            vm.commit_account(AccountCommitment::Full {
                nonce: nonce,
                address: address,
                balance: balance,
                code: code,
            });
        },
        Err(RequireError::AccountStorage(address, index)) => {
            println!("Feeding VM account storage at 0x{:x} with index 0x{:x} ...", address, index);
            let value = M256::from_str(&client.get_storage_at(&format!("0x{:x}", address),
                                                              &format!("0x{:x}", index),
                                                              &block.number)).unwrap();
            vm.commit_account(AccountCommitment::Storage {
                address: address,
                index: index,
                value: value,
            });
        }
        Err(err) => {
            println!("Unhandled require: {:?}", err);
            unimplemented!()
        },
    }
}
```
Notice how a `RequireError` is thrown for `Account` and `AccountStorage`. Some contracts, when executing, need to know more information during execution before it can continue. Typically this information is retrieved from the blockchain on demand basis. The `AccountStorage` is another example of feeding the address of the contract storage so the VM can continue execution.

The rationale behind this design is execution can swiftly continue for most contracts that do not require talking to other contracts or referencing excessive account storage.

Note this API is currently `raw` and thus highly experimental. The possibilty of it changing for a `call` mechanism is higher as integrating a WebAssembly VM is highly probable and thus the semantics will undoubtedly require changing.
