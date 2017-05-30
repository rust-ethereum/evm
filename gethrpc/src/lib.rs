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
struct RPCObjectRequest<T> {
    jsonrpc: String,
    method: String,
    params: T,
    id: usize,
}

#[derive(Serialize, Deserialize)]
struct RPCObjectResponse<T> {
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RPCSyncStatus {
    NotSync(bool),
    Sync {
        startingBlock: String,
        currentBlock: String,
        highestBlock: String,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RPCTransactionReceipt {
    pub transactionHash: String,
    pub transactionIndex: String,
    pub blockHash: String,
    pub blockNumber: String,
    pub cumulativeGasUsed: String,
    pub gasUsed: String,
    pub contractAddress: String,
    pub logs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RPCFilter {
    pub fromBlock: String,
    pub toBlock: String,
    pub address: String,
    pub topics: Vec<String>,
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
        self.rpc_object_request::<Vec<String>, T>(method, params)
    }

    fn rpc_object_request<Req: serde::Serialize, Res: serde::Deserialize>(&mut self, method: &str, params: Req) -> Res {
        let request = RPCObjectRequest {
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

        let response: RPCObjectResponse<Res> = serde_json::from_str(&buffer).unwrap();
        response.result
    }

    pub fn client_version(&mut self) -> String {
        self.rpc_request::<String>("web3_clientVersion", vec![])
    }

    pub fn net_version(&mut self) -> String {
        self.rpc_request::<String>("net_version", vec![])
    }

    pub fn net_listening(&mut self) -> bool {
        self.rpc_request::<bool>("net_listening", vec![])
    }

    pub fn net_peer_count(&mut self) -> String {
        self.rpc_request::<String>("net_peerCount", vec![])
    }

    pub fn sha3(&mut self, data: &str) -> String {
        self.rpc_request::<String>("web3_sha3", vec![])
    }

    pub fn protocol_version(&mut self) -> String {
        self.rpc_request::<String>("eth_protocolVersion", vec![])
    }

    pub fn syncing(&mut self) -> RPCSyncStatus {
        self.rpc_request::<RPCSyncStatus>("eth_syncing", vec![])
    }

    pub fn coinbase(&mut self) -> String {
        self.rpc_request::<String>("eth_coinbase", vec![])
    }

    pub fn mining(&mut self) -> bool {
        self.rpc_request::<bool>("eth_mining", vec![])
    }

    pub fn hashrate(&mut self) -> String {
        self.rpc_request::<String>("eth_hashrate", vec![])
    }

    pub fn gas_price(&mut self) -> String {
        self.rpc_request::<String>("eth_gasPrice", vec![])
    }

    pub fn accounts(&mut self) -> Vec<String> {
        self.rpc_request::<Vec<String>>("eth_accounts", vec![])
    }

    pub fn block_number(&mut self) -> String {
        self.rpc_request::<String>("eth_blockNumber", vec![])
    }

    pub fn get_balance(&mut self, address: &str, number: &str) -> String {
        self.rpc_request::<String>("eth_getBalance",
                                   vec![address.to_string(), number.to_string()])
    }

    pub fn get_storage_at(&mut self, address: &str, index: &str, number: &str) -> String {
        self.rpc_request::<String>("eth_getStorageAt",
                                   vec![address.to_string(), index.to_string(), number.to_string()])
    }

    pub fn get_transaction_count(&mut self, address: &str, number: &str) -> String {
        self.rpc_request::<String>("eth_getTransactionCount",
                                   vec![address.to_string(), number.to_string()])
    }

    pub fn get_block_transaction_count_by_hash(&mut self, hash: &str) -> String {
        self.rpc_request::<String>("eth_getBlockTransactionCountByHash",
                                   vec![hash.to_string()])
    }

    pub fn get_block_transaction_count_by_number(&mut self, number: &str) -> String {
        self.rpc_request::<String>("eth_getBlockTransactionCountByNumber",
                                   vec![number.to_string()])
    }

    pub fn get_uncle_count_by_block_hash(&mut self, hash: &str) -> String {
        self.rpc_request::<String>("eth_getUncleCountByBlockHash",
                                   vec![hash.to_string()])
    }

    pub fn get_uncle_count_by_block_number(&mut self, number: &str) -> String {
        self.rpc_request::<String>("eth_getUncleCountByBlockNumber",
                                   vec![number.to_string()])
    }

    pub fn get_code(&mut self, address: &str, number: &str) -> String {
        self.rpc_request::<String>("eth_getCode", vec![address.to_string(), number.to_string()])
    }

    pub fn sign(&mut self, address: &str, message: &str) -> String {
        self.rpc_request::<String>("eth_sign", vec![address.to_string(), message.to_string()])
    }

    pub fn send_transaction(&mut self, transaction: RPCTransaction) -> String {
        unimplemented!()
    }

    pub fn send_raw_transaction(&mut self, data: &str) -> String {
        self.rpc_request::<String>("eth_sendRawTransaction", vec![data.to_string()])
    }

    pub fn call(&mut self, transaction: RPCTransaction) -> String {
        unimplemented!()
    }

    pub fn estimate_gas(&mut self, transaction: RPCTransaction) -> String {
        unimplemented!()
    }

    pub fn get_block_by_hash(&mut self, hash: &str) -> RPCBlock {
        self.rpc_request::<RPCBlock>("eth_getBlockByHash",
                                     vec![hash.to_string(), "false".to_string()])
    }

    pub fn get_block_by_number(&mut self, number: &str) -> RPCBlock {
        self.rpc_request::<RPCBlock>("eth_getBlockByNumber",
                                     vec![number.to_string(), "false".to_string()])
    }

    pub fn get_transaction_by_hash(&mut self, hash: &str) -> RPCTransaction {
        self.rpc_request::<RPCTransaction>("eth_getTransactionByHash",
                                           vec![hash.to_string()])
    }

    pub fn get_transaction_by_block_hash_and_index(&mut self, hash: &str, index: &str) -> RPCTransaction {
        self.rpc_request::<RPCTransaction>("eth_getTransactionByBlockHashAndIndex",
                                           vec![hash.to_string(), index.to_string()])
    }

    pub fn get_transaction_by_block_number_and_index(&mut self, number: &str, index: &str)
                                                 -> RPCTransaction {
        self.rpc_request::<RPCTransaction>("eth_getTransactionByBlockNumberAndIndex",
                                           vec![number.to_string(), index.to_string()])
    }

    pub fn get_transaction_receipt(&mut self, hash: &str) -> RPCTransactionReceipt {
        self.rpc_request::<RPCTransactionReceipt>("eth_getTransactionReceipt",
                                                 vec![hash.to_string()])
    }

    pub fn get_uncle_by_block_hash_and_index(&mut self, hash: &str, index: &str) -> RPCBlock {
        self.rpc_request::<RPCBlock>("eth_getUncleByBlockHashAndIndex",
                                     vec![hash.to_string(), index.to_string()])
    }

    pub fn get_uncle_by_block_number_and_index(&mut self, number: &str, index: &str) -> RPCBlock {
        self.rpc_request::<RPCBlock>("eth_getUncleByBlockNumberAndIndex",
                                     vec![number.to_string(), index.to_string()])
    }

    pub fn get_compilers(&mut self) -> Vec<String> {
        self.rpc_request::<Vec<String>>("eth_getCompilers", vec![])
    }

    pub fn compile_solidity(&mut self, source: &str) -> String {
        unimplemented!()
    }

    pub fn compile_lll(&mut self, source: &str) -> String {
        self.rpc_request::<String>("eth_compileLLL", vec![source.to_string()])
    }

    pub fn compile_serpent(&mut self, source: &str) -> String {
        self.rpc_request::<String>("eth_compileSerpent", vec![source.to_string()])
    }

    pub fn new_filter(&mut self, filter: RPCFilter) -> String {
        unimplemented!()
    }

    pub fn new_block_filter(&mut self) -> String {
        self.rpc_request::<String>("eth_newBlockFilter", vec![])
    }

    pub fn new_pending_transaction_filter(&mut self) -> String {
        self.rpc_request::<String>("eth_newPendingTransactionFilter", vec![])
    }

    pub fn uninstall_filter(&mut self, id: &str) -> bool {
        self.rpc_request::<bool>("eth_uninstallFilter", vec![id.to_string()])
    }

    pub fn get_filter_changes(&mut self, id: &str) -> String {
        unimplemented!()
    }

    pub fn get_filter_logs(&mut self, id: &str) -> String {
        unimplemented!()
    }

    pub fn get_logs(&mut self, id: &str) -> String {
        unimplemented!()
    }

    pub fn get_work(&mut self) -> Vec<String> {
        self.rpc_request::<Vec<String>>("eth_getWork", vec![])
    }

    pub fn submit_work(&mut self, nonce: &str, pow: &str, mix: &str) -> bool {
        self.rpc_request::<bool>("eth_submitWork",
                                 vec![nonce.to_string(), pow.to_string(), mix.to_string()])
    }

    pub fn submit_hashrate(&mut self, hashrate: &str, id: &str) -> bool {
        self.rpc_request::<bool>("eth_submitHashrate",
                                 vec![hashrate.to_string(), id.to_string()])
    }

    pub fn put_string(&mut self, db: &str, key: &str, value: &str) -> bool {
        self.rpc_request::<bool>("db_putString",
                                 vec![db.to_string(), key.to_string(), value.to_string()])
    }

    pub fn get_string(&mut self, db: &str, key: &str) -> String {
        self.rpc_request::<String>("db_getString",
                                   vec![db.to_string(), key.to_string()])
    }

    pub fn put_hex(&mut self, db: &str, key: &str, value: &str) -> bool {
        self.rpc_request::<bool>("db_putHex",
                                 vec![db.to_string(), key.to_string(), value.to_string()])
    }

    pub fn get_hex(&mut self, db: &str, key: &str) -> String {
        self.rpc_request::<String>("db_getHex",
                                   vec![db.to_string(), key.to_string()])
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
