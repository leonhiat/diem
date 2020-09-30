// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{methods, runtime, tests};
use futures::{channel::mpsc::channel, StreamExt};
use libra_config::config;
use libra_proptest_helpers::ValueGenerator;
use std::sync::Arc;
use warp::reply::Reply;

#[macro_export]
macro_rules! gen_request_params {
    ($params: tt) => {
        serde_json::to_vec(&serde_json::json!($params))
            .expect("failed to convert JSON to byte array")
    };
}

#[test]
fn test_json_rpc_service_fuzzer() {
    let mut gen = ValueGenerator::new();
    let data = generate_corpus(&mut gen);
    fuzzer(&data);
}

#[test]
fn test_method_fuzzer() {
    let params = gen_request_params!([]);
    method_fuzzer(&params, "get_metadata");
}

pub fn method_fuzzer(params_data: &[u8], method: &str) {
    let params = match serde_json::from_slice::<serde_json::Value>(params_data) {
        Err(_) => {
            // should not throw error or panic on invalid fuzzer inputs
            if cfg!(test) {
                panic!();
            }
            return;
        }
        Ok(request) => request,
    };
    let request =
        serde_json::json!({"jsonrpc": "2.0", "method": method, "params": params, "id": 1});
    request_fuzzer(request)
}

/// generate_corpus produces an arbitrary transaction to submit to JSON RPC service
pub fn generate_corpus(gen: &mut ValueGenerator) -> Vec<u8> {
    // use proptest to generate a SignedTransaction
    let txn = gen.generate(proptest::arbitrary::any::<
        libra_types::transaction::SignedTransaction,
    >());
    let payload = hex::encode(lcs::to_bytes(&txn).unwrap());
    let request =
        serde_json::json!({"jsonrpc": "2.0", "method": "submit", "params": [payload], "id": 1});
    serde_json::to_vec(&request).expect("failed to convert JSON to byte array")
}

pub fn fuzzer(data: &[u8]) {
    let json_request = match serde_json::from_slice::<serde_json::Value>(data) {
        Err(_) => {
            // should not throw error or panic on invalid fuzzer inputs
            if cfg!(test) {
                panic!();
            }
            return;
        }
        Ok(request) => request,
    };
    request_fuzzer(json_request)
}

pub fn request_fuzzer(json_request: serde_json::Value) {
    // set up mock Shared Mempool
    let (mp_sender, mut mp_events) = channel(1);

    let db = tests::MockLibraDB {
        version: 1 as u64,
        genesis: std::collections::HashMap::new(),
        all_accounts: std::collections::HashMap::new(),
        all_txns: vec![],
        events: vec![],
        account_state_with_proof: vec![],
        timestamps: vec![1598223353000000],
    };
    let registry = Arc::new(methods::build_registry());
    let service = methods::JsonRpcService::new(
        Arc::new(db),
        mp_sender,
        config::RoleType::Validator,
        libra_types::chain_id::ChainId::test(),
        config::DEFAULT_BATCH_SIZE_LIMIT,
        config::DEFAULT_PAGE_SIZE_LIMIT,
    );
    let mut rt = tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap();

    rt.spawn(async move {
        if let Some((_, cb)) = mp_events.next().await {
            cb.send(Ok((
                libra_types::mempool_status::MempoolStatus::new(
                    libra_types::mempool_status::MempoolStatusCode::Accepted,
                ),
                None,
            )))
            .unwrap();
        }
    });
    let body = rt.block_on(async {
        let reply = runtime::rpc_endpoint(json_request, service, registry)
            .await
            .unwrap();

        let resp = reply.into_response();
        let (_, body) = resp.into_parts();
        hyper::body::to_bytes(body).await.unwrap()
    });

    let response: serde_json::Value = serde_json::from_slice(body.as_ref()).expect("json");

    match response {
        serde_json::Value::Array(batch_response) => {
            for resp in batch_response {
                assert_response(resp)
            }
        }
        _ => assert_response(response),
    };
}

fn assert_response(response: serde_json::Value) {
    let json_rpc_protocol = response.get("jsonrpc");
    assert_eq!(
        json_rpc_protocol,
        Some(&serde_json::Value::String("2.0".to_string())),
        "JSON RPC response with incorrect protocol: {:?}",
        response
    );

    // only proceed to check successful response for input from `generate_corpus`
    if !cfg!(test) {
        return;
    }

    assert!(response.get("result").is_some());
    assert!(response.get("error").is_none());
    let response_id: u64 = serde_json::from_value(
        response
            .get("id")
            .unwrap_or_else(|| panic!("failed to get ID from response: {}", response))
            .clone(),
    )
    .unwrap_or_else(|_| panic!("Failed to deserialize ID from: {}", response));
    assert_eq!(response_id, 1, "mismatch ID in JSON RPC: {}", response);
}
