#[macro_use]
extern crate clap;
extern crate bigint;
extern crate hexutil;
extern crate sputnikvm;
extern crate serde_json;
extern crate gethrpc;
extern crate flame;

mod profiler;

use std::fs::File;

use bigint::{Gas, Address, U256, M256, H256};
use hexutil::read_hex;
use sputnikvm::{HeaderParams, Context, SeqTransactionVM, ValidTransaction, VM, Log, Patch,
                AccountCommitment, AccountChange, RequireError, TransactionAction, VMStatus,
                MainnetFrontierPatch, MainnetHomesteadPatch, MainnetEIP150Patch, MainnetEIP160Patch,
                SeqContextVM};
use gethrpc::{GethRPCClient, NormalGethRPCClient, RPCBlock};
use std::str::FromStr;
use std::ops::DerefMut;
use std::rc::Rc;

fn from_rpc_block(block: &RPCBlock) -> HeaderParams {
    HeaderParams {
        beneficiary: Address::from_str(&block.miner).unwrap(),
        timestamp: U256::from_str(&block.timestamp).unwrap().as_u64(),
        number: U256::from_str(&block.number).unwrap(),
        difficulty: U256::from_str(&block.difficulty).unwrap(),
        gas_limit: Gas::from_str(&block.gasLimit).unwrap(),
    }
}

fn handle_step_without_rpc(vm: &mut VM) {
    match vm.step() {
        Ok(()) => {},
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
            vm.commit_account(AccountCommitment::Nonexist(address)).unwrap();
        },
        Err(RequireError::Blockhash(number)) => {
            vm.commit_blockhash(number, H256::default()).unwrap();
        },
    }
}

fn profile_fire_without_rpc(vm: &mut VM) {
    loop {
        match vm.status() {
            VMStatus::Running => {
                let instruction = vm.peek();
                flame::span_of(format!("{:?}", instruction), || {
                    handle_step_without_rpc(vm)
                });
            },
            VMStatus::ExitedOk | VMStatus::ExitedErr(_) |
            VMStatus::ExitedNotSupported(_) => return,
        }
    }
}

fn handle_fire_without_rpc(vm: &mut VM) {
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
                vm.commit_account(AccountCommitment::Nonexist(address)).unwrap();
            },
            Err(RequireError::Blockhash(number)) => {
                vm.commit_blockhash(number, H256::default()).unwrap();
            },
        }
    }
}

fn handle_fire_with_rpc<T: GethRPCClient>(client: &mut T, vm: &mut VM, block_number: &str) {
    loop {
        match vm.fire() {
            Ok(()) => break,
            Err(RequireError::Account(address)) => {
                let nonce = U256::from_str(&client.get_transaction_count(&format!("0x{:x}", address),
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
                        code: Rc::new(code),
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
                    code: Rc::new(code),
                }).unwrap();
            },
            Err(RequireError::Blockhash(number)) => {
                let hash = H256::from_str(&client.get_block_by_number(&format!("0x{:x}", number))
                    .hash).unwrap();
                vm.commit_blockhash(number, hash).unwrap();
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
        (@arg PROFILE: --profile "Whether to output a profiling result for the execution.")
        (@arg CODE: --code +takes_value +required "Code to be executed.")
        (@arg RPC: --rpc +takes_value "Indicate this EVM should be run on an actual blockchain.")
        (@arg DATA: --data +takes_value "Data associated with this transaction.")
        (@arg BLOCK: --block +takes_value "Block number associated.")
        (@arg PATCH: --patch +takes_value +required "Patch to be used.")
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
    let block_number = matches.value_of("BLOCK").unwrap_or("0x0");

    let block = if matches.is_present("RPC") {
        let mut client = NormalGethRPCClient::new(matches.value_of("RPC").unwrap());
        from_rpc_block(&client.get_block_by_number(block_number))
    } else {
        HeaderParams {
            beneficiary: Address::default(),
            timestamp: 0,
            number: U256::from_str(block_number).unwrap(),
            difficulty: U256::zero(),
            gas_limit: Gas::zero(),
        }
    };

    let mut client = if matches.is_present("RPC") {
        Some(NormalGethRPCClient::new(matches.value_of("RPC").unwrap()))
    } else {
        None
    };

    let mut vm: Box<VM> = if matches.is_present("CODE") {
        let context = Context {
            address, caller, gas_limit, gas_price, value,
            code: Rc::new(code),
            data: Rc::new(data),
            origin: caller,
            apprent_value: value,
            is_system: false,
        };

        match matches.value_of("PATCH") {
            Some("frontier") => Box::new(SeqContextVM::<MainnetFrontierPatch>::new(context, block)),
            Some("homestead") => Box::new(SeqContextVM::<MainnetHomesteadPatch>::new(context, block)),
            Some("eip150") => Box::new(SeqContextVM::<MainnetEIP150Patch>::new(context, block)),
            Some("eip160") => Box::new(SeqContextVM::<MainnetEIP160Patch>::new(context, block)),
            _ => panic!("Unsupported patch."),
        }
    } else {
        let transaction = ValidTransaction {
            caller: Some(caller),
            value, gas_limit, gas_price,
            input: Rc::new(data),
            nonce: match client {
                Some(ref mut client) => {
                    U256::from_str(&client.get_transaction_count(&format!("0x{:x}", caller),
                                                                 &block_number)).unwrap()
                },
                None => U256::zero(),
            },
            action: if is_create {
                TransactionAction::Create
            } else {
                TransactionAction::Call(address)
            },
        };

        match matches.value_of("PATCH") {
            Some("frontier") => Box::new(SeqTransactionVM::<MainnetFrontierPatch>::new(transaction, block)),
            Some("homestead") => Box::new(SeqTransactionVM::<MainnetHomesteadPatch>::new(transaction, block)),
            Some("eip150") => Box::new(SeqTransactionVM::<MainnetEIP150Patch>::new(transaction, block)),
            Some("eip160") => Box::new(SeqTransactionVM::<MainnetEIP160Patch>::new(transaction, block)),
            _ => panic!("Unsupported patch."),
        }
    };
    match client {
        Some(ref mut client) => {
            handle_fire_with_rpc(client, vm.deref_mut(), block_number);
        },
        None => {
            if matches.is_present("PROFILE") {
                profile_fire_without_rpc(vm.deref_mut());
                flame::dump_html(&mut File::create("profile.html").unwrap()).unwrap();
            } else {
                handle_fire_without_rpc(vm.deref_mut());
            }
        },
    }

    println!("VM returned: {:?}", vm.status());
    println!("VM out: {:?}", vm.out());
    for account in vm.accounts() {
        println!("{:?}", account);
    }
}
