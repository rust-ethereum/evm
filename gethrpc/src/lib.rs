extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate hyper_native_tls;

#[macro_use]
extern crate serde_derive;

mod record;

pub use record::{RecordGethRPCClient, CachedGethRPCClient};

use std::io::Read;
use hyper::header::ContentType;
use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

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
#[serde(rename_all = "camelCase")]
pub struct RPCTransaction {
    pub hash: String,
    pub nonce: String,
    pub block_hash: String,
    pub block_number: String,
    pub transaction_index: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: String,
    pub gas_price: String,
    pub input: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RPCCall {
    pub from: String,
    pub to: String,
    pub gas: String,
    pub gas_price: String,
    pub value: String,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RPCBlock {
    pub number: String,
    pub hash: String,
    pub parent_hash: String,
    pub nonce: String,
    pub sha3_uncles: String,
    pub logs_bloom: String,
    pub transactions_root: String,
    pub state_root: String,
    pub miner: String,
    pub difficulty: String,
    pub total_difficulty: String,
    pub extra_data: String,
    pub size: String,
    pub gas_limit: String,
    pub gas_used: String,
    pub timestamp: String,
    #[serde(default)]
    pub transactions: Vec<String>,
    pub uncles: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum RPCSyncStatus {
    NotSync(bool),
    Sync {
        starting_block: String,
        current_block: String,
        highest_block: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RPCTransactionReceipt {
    pub transaction_hash: String,
    pub transaction_index: String,
    pub block_hash: String,
    pub block_number: String,
    pub cumulative_gas_used: String,
    pub gas_used: String,
    pub contract_address: String,
    pub logs: Vec<RPCLog>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RPCLog {
    pub log_index: String,
    pub transaction_index: String,
    pub transaction_hash: String,
    pub block_hash: String,
    pub block_number: String,
    pub address: String,
    pub data: String,
    pub topics: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RPCFilter {
    pub from_block: String,
    pub to_block: String,
    pub address: String,
    pub topics: Vec<String>,
}

pub trait GethRPCClient {
    fn rpc_object_request<Req: serde::Serialize, Res: serde::Deserialize>(
        &mut self,
        method: &str,
        params: Req,
    ) -> Res;

    fn rpc_request<T: serde::Deserialize>(&mut self, method: &str, params: Vec<String>) -> T {
        self.rpc_object_request::<Vec<String>, T>(method, params)
    }

    fn client_version(&mut self) -> String {
        self.rpc_request::<String>("web3_clientVersion", vec![])
    }
    fn net_version(&mut self) -> String {
        self.rpc_request::<String>("net_version", vec![])
    }
    fn net_listening(&mut self) -> bool {
        self.rpc_request::<bool>("net_listening", vec![])
    }
    fn net_peer_count(&mut self) -> String {
        self.rpc_request::<String>("net_peerCount", vec![])
    }
    fn sha3(&mut self, data: &str) -> String {
        self.rpc_request::<String>("web3_sha3", vec![data.to_string()])
    }
    fn protocol_version(&mut self) -> String {
        self.rpc_request::<String>("eth_protocolVersion", vec![])
    }
    fn syncing(&mut self) -> RPCSyncStatus {
        self.rpc_request::<RPCSyncStatus>("eth_syncing", vec![])
    }
    fn coinbase(&mut self) -> String {
        self.rpc_request::<String>("eth_coinbase", vec![])
    }
    fn mining(&mut self) -> bool {
        self.rpc_request::<bool>("eth_mining", vec![])
    }
    fn hashrate(&mut self) -> String {
        self.rpc_request::<String>("eth_hashrate", vec![])
    }
    fn gas_price(&mut self) -> String {
        self.rpc_request::<String>("eth_gasPrice", vec![])
    }
    fn accounts(&mut self) -> Vec<String> {
        self.rpc_request::<Vec<String>>("eth_accounts", vec![])
    }
    fn block_number(&mut self) -> String {
        self.rpc_request::<String>("eth_blockNumber", vec![])
    }

    fn account_exist(&mut self, address: &str, number: usize) -> bool {
        self.rpc_object_request::<(String, usize), bool>(
            "debug_accountExist",
            (address.to_string(), number),
        )
    }

    fn get_balance(&mut self, address: &str, number: &str) -> String {
        self.rpc_request::<String>(
            "eth_getBalance",
            vec![address.to_string(), number.to_string()],
        )
    }

    fn get_storage_at(&mut self, address: &str, index: &str, number: &str) -> String {
        self.rpc_request::<String>(
            "eth_getStorageAt",
            vec![address.to_string(), index.to_string(), number.to_string()],
        )
    }

    fn get_transaction_count(&mut self, address: &str, number: &str) -> String {
        self.rpc_request::<String>(
            "eth_getTransactionCount",
            vec![address.to_string(), number.to_string()],
        )
    }

    fn get_block_transaction_count_by_hash(&mut self, hash: &str) -> String {
        self.rpc_request::<String>("eth_getBlockTransactionCountByHash", vec![hash.to_string()])
    }

    fn get_block_transaction_count_by_number(&mut self, number: &str) -> String {
        self.rpc_request::<String>(
            "eth_getBlockTransactionCountByNumber",
            vec![number.to_string()],
        )
    }

    fn get_uncle_count_by_block_hash(&mut self, hash: &str) -> String {
        self.rpc_request::<String>("eth_getUncleCountByBlockHash", vec![hash.to_string()])
    }

    fn get_uncle_count_by_block_number(&mut self, number: &str) -> String {
        self.rpc_request::<String>("eth_getUncleCountByBlockNumber", vec![number.to_string()])
    }

    fn get_code(&mut self, address: &str, number: &str) -> String {
        self.rpc_request::<String>("eth_getCode", vec![address.to_string(), number.to_string()])
    }

    fn send_raw_transaction(&mut self, data: &str) -> String {
        self.rpc_request::<String>("eth_sendRawTransaction", vec![data.to_string()])
    }

    fn call(&mut self, transaction: RPCCall, number: &str) -> String {
        self.rpc_object_request::<(RPCCall, String), String>(
            "eth_call",
            (transaction, number.to_string()),
        )
    }

    fn get_block_by_hash(&mut self, hash: &str) -> RPCBlock {
        self.rpc_object_request::<(String, bool), RPCBlock>(
            "eth_getBlockByHash",
            (hash.to_string(), false),
        )
    }

    fn get_block_by_number(&mut self, number: &str) -> RPCBlock {
        self.rpc_object_request::<(String, bool), RPCBlock>(
            "eth_getBlockByNumber",
            (number.to_string(), false),
        )
    }

    fn get_transaction_by_hash(&mut self, hash: &str) -> RPCTransaction {
        self.rpc_request::<RPCTransaction>("eth_getTransactionByHash", vec![hash.to_string()])
    }

    fn get_transaction_by_block_hash_and_index(
        &mut self,
        hash: &str,
        index: &str,
    ) -> RPCTransaction {
        self.rpc_request::<RPCTransaction>(
            "eth_getTransactionByBlockHashAndIndex",
            vec![hash.to_string(), index.to_string()],
        )
    }

    fn get_transaction_by_block_number_and_index(
        &mut self,
        number: &str,
        index: &str,
    ) -> RPCTransaction {
        self.rpc_request::<RPCTransaction>(
            "eth_getTransactionByBlockNumberAndIndex",
            vec![number.to_string(), index.to_string()],
        )
    }

    fn get_transaction_receipt(&mut self, hash: &str) -> RPCTransactionReceipt {
        self.rpc_request::<RPCTransactionReceipt>(
            "eth_getTransactionReceipt",
            vec![hash.to_string()],
        )
    }

    fn get_uncle_by_block_hash_and_index(&mut self, hash: &str, index: &str) -> RPCBlock {
        self.rpc_request::<RPCBlock>(
            "eth_getUncleByBlockHashAndIndex",
            vec![hash.to_string(), index.to_string()],
        )
    }

    fn get_uncle_by_block_number_and_index(&mut self, number: &str, index: &str) -> RPCBlock {
        self.rpc_request::<RPCBlock>(
            "eth_getUncleByBlockNumberAndIndex",
            vec![number.to_string(), index.to_string()],
        )
    }
}

pub struct NormalGethRPCClient {
    endpoint: String,
    free_id: usize,
    http: Client,
}

impl NormalGethRPCClient {
    pub fn new(endpoint: &str) -> Self {
        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);
        NormalGethRPCClient {
            endpoint: endpoint.to_string(),
            free_id: 1,
            http: Client::with_connector(connector),
        }
    }
}

impl GethRPCClient for NormalGethRPCClient {
    fn rpc_object_request<Req: serde::Serialize, Res: serde::Deserialize>(
        &mut self,
        method: &str,
        params: Req,
    ) -> Res {
        let request = RPCObjectRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: params,
            id: self.free_id,
        };
        self.free_id = self.free_id + 1;

        let mut response_raw = self.http
            .post(&self.endpoint)
            .header(ContentType::json())
            .body(&serde_json::to_string(&request).unwrap())
            .send()
            .unwrap();
        let mut buffer = String::new();
        response_raw.read_to_string(&mut buffer).unwrap();

        let response: RPCObjectResponse<Res> = serde_json::from_str(&buffer).unwrap();
        response.result
    }
}
