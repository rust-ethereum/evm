extern crate serde_json;

use std::process::{Command};
use serde_json::Value;

fn code(tx_to: &str, tx_from: &str, blk_no: &str, address: &str) {
    let prefix = "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getCode\",\"params\":[\"";
    let postfix = "\"],\"id\":1}";
    let together = format!("{}{}\", \"{}{}", prefix, tx_to, blk_no, postfix);
    let mut cmd = Command::new("curl")
        .arg("-X")
        .arg("POST")
        .arg("--data")
        .arg(together)
        .arg(address)
        .output()
        .unwrap();
    let serialized = String::from_utf8_lossy(&cmd.stdout);
    let v: Value = serde_json::from_str(&serialized).unwrap();
    match v["result"].as_str() {
        Some(code) => {
            match code {
                "0x" => {},
                _ => {
                    let blk_no = i64::from_str_radix(&blk_no[2..], 16);
                    print!("blk_no: {:?}, code: {:?}\n", blk_no, code );
                }
            }
        },
        None => {},
    };
}

fn transaction_by_block_number_and_index(blk_no: &str, tx_index: &str, address: &str) {
    let prefix = "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getTransactionByBlockNumberAndIndex\",\"params\":[\"";
    let postfix = "\"],\"id\":1}";
    let together = format!("{}{}\",\"{}{}", prefix, blk_no, tx_index, postfix);
    let mut cmd = Command::new("curl")
        .arg("-X")
        .arg("POST")
        .arg("--data")
        .arg(together)
        .arg(address)
        .output()
        .unwrap();
    let serialized = String::from_utf8_lossy(&cmd.stdout);
    let v: Value = serde_json::from_str(&serialized).unwrap();
    let to = match v["result"]["to"].as_str() {
        Some(s) => {s},
        None => { return }, // if it's null it means a contract was created
    };
    let from = v["result"]["from"].as_str().unwrap();
    let blk_no = v["result"]["blockNumber"].as_str().unwrap();
    code(to, from, blk_no, address);
}

fn transaction_count(blk_no: &str, address: &str) {
    let prefix = "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockTransactionCountByNumber\",\"params\":[\"";
    let postfix = "\"],\"id\":1}";
    let together = format!("{}{}{}", prefix, blk_no, postfix);
    let mut cmd = Command::new("curl")
        .arg("-X")
        .arg("POST")
        .arg("--data")
        .arg(together)
        .arg(address)
        .output()
        .unwrap();
    let serialized = String::from_utf8_lossy(&cmd.stdout);
    let v: Value = serde_json::from_str(&serialized).unwrap();
    match v["result"].as_str() {
        Some(tx_count) => {
            match tx_count {
                "0x0" => {},
                _ => {
                    let tx_count = i64::from_str_radix(&tx_count[2..], 16);
                    for index in 0..tx_count.unwrap() {
                        transaction_by_block_number_and_index(blk_no, format!("0x{:x}", index).as_str(), address)
                    }
                }
            }
        },
        None => {},
    };
}

fn block(blk_no: &str, address: &str) {
    let prefix = "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"";
    let postfix = "\", true],\"id\":1}";
    let together = format!("{}{}{}", prefix, blk_no, postfix);
    let mut cmd = Command::new("curl")
        .arg("-X")
        .arg("POST")
        .arg("--data")
        .arg(together)
        .arg(address)
        .output()
        .unwrap();
    let serialized = String::from_utf8_lossy(&cmd.stdout);
    let v: Value = serde_json::from_str(&serialized).unwrap();
    match v["result"]["transactionsRoot"].as_str() {
        Some(p) => {
            transaction_count(blk_no, address);
        },
        None => {},
    }
}

fn height(address: &str) -> i64 {
    let mut cmd = Command::new("curl")
        .arg("-X")
        .arg("POST")
        .arg("--data")
        .arg("{\"jsonrpc\":\"2.0\",\"method\":\"eth_blockNumber\",\"params\":[],\"id\":83}")
        .arg(address)
        .output()
        .unwrap();
    let serialized = String::from_utf8_lossy(&cmd.stdout);
    let v: Value = serde_json::from_str(&serialized).unwrap();
    let raw = v["result"].as_str().unwrap();
    let z = i64::from_str_radix(&raw[2..], 16);
    z.unwrap()
}

pub fn regression(address: &str) -> bool {
    let height = height(address);
    println!("height: {}", height);
    // blk_no 49439 is when transactions to code starts appearing.
    for blk_no in 49439..height {
        block(format!("0x{:x}", blk_no).as_str(), address);
    };
    false
}
