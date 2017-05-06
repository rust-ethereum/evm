use serde_json;
use sputnikvm;

use sputnikvm::{read_hex, Gas, M256, Address};
use sputnikvm::vm::{Machine, Stack, PC};

use super::json::{create_machine, create_block, test_machine, apply_to_block, fire_with_block};

use serde_json::{Value, Error};
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Write, stdout};
use std::str::FromStr;

use rustyline::error::ReadlineError;
use rustyline::Editor;

pub fn test_transaction(name: &str, v: &Value, debug: bool) -> bool {
    print!("Testing {} ... ", name);
    if debug {
        print!("\n");
    }
    stdout().flush();

    let mut block = create_block(v);
    let mut machine = create_machine(v, &block);
    let result = fire_with_block(&mut machine, &block);
    apply_to_block(&machine, &mut block);

    let out = v["out"].as_str();

    if out.is_some() {
        if result.is_ok() {
            if test_machine(v, &machine, &block, debug) {
                println!("OK");
                return true;
            } else {
                println!("Failed (result not match)");
                return false;
            }
        } else {
            println!("Failed {:?}", result.err().unwrap());
            return false;
        }
    } else {
        if result.is_err() {
            println!("OK");
            return true;
        } else {
            println!("Failed");
            return false;
        }
    }
}


pub fn debug_transaction(v: &Value) {
    let mut block = create_block(v);
    let mut machine = create_machine(v, &block);
    let owner = machine.owner().unwrap();
    machine.commit(block.request_account(owner));
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                match line.as_ref() {
                    "step" => {
                        if machine.pc().unwrap().stopped() {
                            println!("Stopped");
                        } else {
                            println!("Running {:?} ... {:?}.", machine.pc().unwrap().peek_opcode(),
                                     machine.step());
                        }
                    },
                    "fire" => {
                        let result = machine.fire();
                        println!("{:?}", result);
                    },
                    "fire debug" => {
                        while !machine.pc().unwrap().stopped() {
                            println!("Running {:?} ...", machine.pc().unwrap().peek_opcode());
                            let gas = machine.peek_cost().unwrap();
                            if gas < Gas::from(u64::max_value()) {
                                let gas: u64 = gas.into();
                                println!("Cost: {}", gas);
                            } else {
                                println!("Cost: 0x{:x}", gas);
                            }
                            for i in 0..machine.stack().len() {
                                println!("{}: {:x}", i, machine.stack().peek(i).unwrap());
                            }
                            println!("Result: {:?}", machine.step());
                            print!("\n");
                        }
                    },
                    "gas" => {
                        let gas = machine.peek_cost();
                        if gas.is_ok() {
                            println!("0x{:x}", gas.unwrap());
                        } else {
                            println!("{:?}", gas);
                        }
                    }
                    "out" => {
                        let ret = machine.return_values();
                        println!("{:?}", ret);
                    }
                    "print stack" => {
                        for i in 0..machine.stack().len() {
                            println!("{}: {:x}", i, machine.stack().peek(i).unwrap());
                        }
                    },
                    _ => {
                        println!("Unknown command.");
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
}
