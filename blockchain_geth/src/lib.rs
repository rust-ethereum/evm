extern crate serde;
extern crate serde_json;
extern crate hyper;

#[macro_use]
extern crate serde_derive;

use std::process::{Command};
use std::io::Read;
use hyper::header::ContentType;
use hyper::client::Client;
use serde_json::Value;

#[derive(Serialize, Deserialize)]
struct RPCRequest {
    jsonrpc: String,
    method: String,
    params: Vec<String>,
    id: usize,
}

#[derive(Serialize, Deserialize)]
struct RPCResponse<T> {
    jsonrpc: String,
    result: T,
    id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RPCTransaction {
    pub hash: String,
    pub nonce: String,
    pub blockHash: String,
    pub blockNumber: String,
    pub transactionIndex: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: String,
    pub gasPrice: String,
    pub input: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RPCBlock {
    pub number: String,
    pub hash: String,
    pub parentHash: String,
    pub nonce: String,
    pub sha3Uncles: String,
    pub logsBloom: String,
    pub transactionsRoot: String,
    pub stateRoot: String,
    pub miner: String,
    pub difficulty: String,
    pub totalDifficulty: String,
    pub extraData: String,
    pub size: String,
    pub gasLimit: String,
    pub gasUsed: String,
    pub timestamp: String,
    pub transactions: Vec<String>,
    pub uncles: Vec<String>
}

pub struct GethRPCClient {
    endpoint: String,
    free_id: usize,
}

impl GethRPCClient {
    pub fn new(endpoint: &str) -> Self {
        GethRPCClient {
            endpoint: endpoint.to_string(),
            free_id: 1,
        }
    }

    fn rpc_request<T: serde::Deserialize>(&mut self, method: &str, params: Vec<String>) -> T {
        let request = RPCRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: params,
            id: self.free_id,
        };
        self.free_id = self.free_id + 1;

        let client = Client::new();
        let mut response_raw = client.post(&self.endpoint)
            .header(ContentType::json())
            .body(&serde_json::to_string(&request).unwrap())
            .send().unwrap();
        let mut buffer = String::new();
        response_raw.read_to_string(&mut buffer).unwrap();

        let response: RPCResponse<T> = serde_json::from_str(&buffer).unwrap();
        response.result
    }

    pub fn get_block_transaction_count_by_number(&mut self, number: &str) -> String {
        self.rpc_request::<String>("eth_getBlockTransactionCountByNumber",
                                   vec![number.to_string()])
    }

    pub fn get_transaction_by_block_number_and_index(&mut self, number: &str, index: &str)
                                                 -> RPCTransaction {
        self.rpc_request::<RPCTransaction>("eth_getTransactionByBlockNumberAndIndex",
                                           vec![number.to_string(), index.to_string()])
    }

    pub fn get_block_by_number(&mut self, number: &str) -> RPCBlock {
        self.rpc_request::<RPCBlock>("eth_getBlockByNumber",
                                     vec![number.to_string(), "false".to_string()])
    }

    pub fn block_number(&mut self) -> String {
        self.rpc_request::<String>("eth_blockNumber", vec![])
    }

    pub fn get_code(&mut self, address: &str, number: &str) -> String {
        self.rpc_request::<String>("eth_getCode", vec![address.to_string(), number.to_string()])
    }
}

pub fn regression(address: &str) -> bool {
    let mut client = GethRPCClient::new("http://localhost:8545");
    let height = usize::from_str_radix(&client.block_number(), 16).unwrap();
    println!("height: {}", height);

    for blk_no in 49439..height {
        let block = client.get_block_by_number(format!("0x{:x}", blk_no).as_str());
        println!("{:?}", block);
    }
    false
}
