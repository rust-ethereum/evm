#[macro_use]
extern crate clap;
extern crate sputnikvm;
extern crate serde_json;
extern crate gethrpc;

use sputnikvm::{Gas, Address, U256, M256, read_hex};
use sputnikvm::vm::{BlockHeader, Context, SeqTransactionVM, Transaction, VM, Log, Patch,
                    AccountCommitment, Account, FRONTIER_PATCH, HOMESTEAD_PATCH};
use sputnikvm::vm::errors::RequireError;
use gethrpc::{GethRPCClient, NormalGethRPCClient, RPCBlock};
use std::str::FromStr;

fn from_rpc_block(block: &RPCBlock) -> BlockHeader {
    BlockHeader {
        coinbase: Address::from_str(&block.miner).unwrap(),
        timestamp: M256::from_str(&block.timestamp).unwrap(),
        number: M256::from_str(&block.number).unwrap(),
        difficulty: M256::from_str(&block.difficulty).unwrap(),
        gas_limit: Gas::from_str(&block.gasLimit).unwrap(),
    }
}

fn handle_fire_without_rpc(vm: &mut SeqTransactionVM) {
    loop {
        match vm.fire() {
            Ok(()) => break,
            Err(RequireError::Account(address)) => {
                vm.commit_account(AccountCommitment::Nonexist(address)).unwrap();
            },
            Err(RequireError::AccountStorage(address, index)) => {
                vm.commit_account(AccountCommitment::Storage {
                    address: address,
                    index: index,
                    value: M256::zero(),
                }).unwrap();
            },
            Err(RequireError::AccountCode(address)) => {
                vm.commit_account(AccountCommitment::Code {
                    address: address,
                    code: Vec::new(),
                }).unwrap();
            },
            Err(RequireError::Blockhash(number)) => {
                vm.commit_blockhash(number, M256::zero());
            },
        }
    }
}

fn handle_fire_with_rpc<T: GethRPCClient>(client: &mut T, vm: &mut SeqTransactionVM, block_number: &str) {
    loop {
        match vm.fire() {
            Ok(()) => break,
            Err(RequireError::Account(address)) => {
                let nonce = M256::from_str(&client.get_transaction_count(&format!("0x{:x}", address),
                                                                         &block_number)).unwrap();
                let balance = U256::from_str(&client.get_balance(&format!("0x{:x}", address),
                                                                 &block_number)).unwrap();
                let code = read_hex(&client.get_code(&format!("0x{:x}", address),
                                                     &block_number)).unwrap();
                if !client.account_exist(&format!("0x{:x}", address), U256::from_str(&block_number).unwrap().as_usize()) {
                    vm.commit_account(AccountCommitment::Nonexist(address)).unwrap();
                } else {
                    vm.commit_account(AccountCommitment::Full {
                        nonce: nonce,
                        address: address,
                        balance: balance,
                        code: code,
                    }).unwrap();
                }
            },
            Err(RequireError::AccountStorage(address, index)) => {
                let value = M256::from_str(&client.get_storage_at(&format!("0x{:x}", address),
                                                                  &format!("0x{:x}", index),
                                                                  &block_number)).unwrap();
                vm.commit_account(AccountCommitment::Storage {
                    address: address,
                    index: index,
                    value: value,
                }).unwrap();
            },
            Err(RequireError::AccountCode(address)) => {
                let code = read_hex(&client.get_code(&format!("0x{:x}", address),
                                                     &block_number)).unwrap();
                vm.commit_account(AccountCommitment::Code {
                    address: address,
                    code: code,
                }).unwrap();
            },
            Err(RequireError::Blockhash(number)) => {
                let hash = M256::from_str(&client.get_block_by_number(&format!("0x{:x}", number))
                    .hash).unwrap();
                vm.commit_blockhash(number, hash);
            },
        }
    }
}

fn main() {
    let matches = clap_app!(sputnikvm =>
        (version: "0.1")
        (author: "Ethereum Classic Contributors")
        (about: "CLI tool for SputnikVM.")
        (@arg CREATE: --create "Execute a CreateContract transaction instead of message call.")
        (@arg CODE: --code +takes_value +required "Code to be executed.")
        (@arg RPC: --rpc +takes_value "Indicate this EVM should be run on an actual blockchain.")
        (@arg DATA: --data +takes_value "Data associated with this transaction.")
        (@arg BLOCK: --block +takes_value "Block number associated.")
        (@arg PATCH: --patch +takes_value "Patch to be used.")
        (@arg GAS_LIMIT: --gas_limit +takes_value "Gas limit.")
        (@arg GAS_PRICE: --gas_price +takes_value "Gas price.")
        (@arg CALLER: --caller +takes_value "Caller of the transaction.")
        (@arg ADDRESS: --address +takes_value "Address of the transaction.")
        (@arg VALUE: --value +takes_value "Value of the transaction.")
    ).get_matches();

    let code = read_hex(matches.value_of("CODE").unwrap()).unwrap();
    let data = read_hex(matches.value_of("DATA").unwrap_or("")).unwrap();
    let caller = Address::from_str(matches.value_of("CALLER").unwrap_or("0x0000000000000000000000000000000000000000")).unwrap();
    let address = Address::from_str(matches.value_of("ADDRESS").unwrap_or("0x0000000000000000000000000000000000000000")).unwrap();
    let value = U256::from_str(matches.value_of("VALUE").unwrap_or("0x0")).unwrap();
    let gas_limit = Gas::from_str(matches.value_of("GAS_LIMIT").unwrap_or("0x2540be400")).unwrap();
    let gas_price = Gas::from_str(matches.value_of("GAS_PRICE").unwrap_or("0x0")).unwrap();
    let is_create = matches.is_present("CREATE");
    let patch = match matches.value_of("PATCH") {
        None => &FRONTIER_PATCH,
        Some("frontier") => &FRONTIER_PATCH,
        Some("homestead") => &HOMESTEAD_PATCH,
        _ => panic!("Unsupported patch."),
    };
    let block_number = matches.value_of("BLOCK").unwrap_or("0x0");

    let block = if matches.is_present("RPC") {
        let mut client = NormalGethRPCClient::new(matches.value_of("RPC").unwrap());
        from_rpc_block(&client.get_block_by_number(block_number))
    } else {
        BlockHeader {
            coinbase: Address::default(),
            timestamp: M256::zero(),
            number: M256::from_str(block_number).unwrap(),
            difficulty: M256::zero(),
            gas_limit: Gas::zero(),
        }
    };

    let transaction = if is_create {
        Transaction::ContractCreation {
            caller, value, gas_limit, gas_price,
            init: data,
        }
    } else {
        Transaction::MessageCall {
            caller, address, value, gas_limit, gas_price, data
        }
    };

    let mut vm = SeqTransactionVM::new(transaction, block, patch);
    if matches.is_present("RPC") {
        let mut client = NormalGethRPCClient::new(matches.value_of("RPC").unwrap());
        handle_fire_with_rpc(&mut client, &mut vm, block_number);
    } else {
        handle_fire_without_rpc(&mut vm);
    }

    println!("VM returned: {:?}", vm.status());
    println!("VM out: {:?}", vm.out());
}
