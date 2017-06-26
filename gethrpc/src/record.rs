use serde_json::Value;
use super::{GethRPCClient, RPCObjectRequest, RPCObjectResponse};

use serde;
use serde_json;
use std::io::Read;
use hyper::header::ContentType;
use hyper::client::Client;

#[derive(Serialize, Deserialize)]
struct Record {
    pub method: String,
    pub request: serde_json::Value,
    pub response: serde_json::Value,
}

pub struct RecordGethRPCClient {
    endpoint: String,
    free_id: usize,
    http: Client,
    records: Vec<Record>,
}

impl RecordGethRPCClient {
    pub fn new(endpoint: &str) -> Self {
        RecordGethRPCClient {
            endpoint: endpoint.to_string(),
            free_id: 1,
            http: Client::new(),
            records: Vec::new(),
        }
    }
}

impl GethRPCClient for RecordGethRPCClient {
    fn rpc_object_request<Req: serde::Serialize, Res: serde::Deserialize>(&mut self, method: &str, params: Req) -> Res {
        let request_value = serde_json::to_value(params).unwrap();
        let request = RPCObjectRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: request_value.clone(),
            id: self.free_id,
        };
        self.free_id = self.free_id + 1;

        let mut response_raw = self.http.post(&self.endpoint)
            .header(ContentType::json())
            .body(&serde_json::to_string(&request).unwrap())
            .send().unwrap();
        let mut buffer = String::new();
        response_raw.read_to_string(&mut buffer).unwrap();

        let response_value: serde_json::Value = serde_json::from_str(&buffer).unwrap();
        let response: RPCObjectResponse<Res> = serde_json::from_value(response_value.clone()).unwrap();

        self.records.push(Record {
            method: method.to_string(),
            request: request_value,
            response: response_value,
        });

        response.result
    }
}

pub struct CachedGethRPCClient {
    records: Vec<Record>,
}

impl CachedGethRPCClient {
    pub fn from_value(value: serde_json::Value) -> Self {
        CachedGethRPCClient { records: serde_json::from_value(value).unwrap() }
    }
}

impl GethRPCClient for CachedGethRPCClient {
    fn rpc_object_request<Req: serde::Serialize, Res: serde::Deserialize>(&mut self, method: &str, params: Req) -> Res {
        let request_value = serde_json::to_value(params).unwrap();

        for record in &self.records {
            if &record.method == method && record.request == request_value {
                let response: RPCObjectResponse<Res> = serde_json::from_value(record.response.clone()).unwrap();
                return response.result;
            }
        }

        panic!()
    }
}
