// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::async_client::{Client, Error, JsonRpcResponse, Retry, State, WaitForTransactionError};
use libra_types::{account_address::AccountAddress, transaction::SignedTransaction};
use serde_json::{json, to_value, Value};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use warp::Filter;

macro_rules! assert_err {
    ($result: expr, $expected: pat, $has_source: expr) => {
        let ret = $result;

        match ret {
            Err(err) => {
                // support display
                assert_ne!("", format!("{}", &err));
                // support debug
                assert_ne!("", format!("{:?}", &err));
                assert_error_source(&err, $has_source);

                match err {
                    $expected => {}
                    _ => panic!("unexpected error {:#?}", err),
                }
            }
            Ok(_) => panic!("no error returned"),
        };
    };
}

fn setup(request: (&'static str, Value), response: Value) -> Client<Retry> {
    setup_batch(vec![(request, response)])
}

fn setup_batch(inouts: Vec<((&'static str, Value), Value)>) -> Client<Retry> {
    let id = Arc::new(AtomicUsize::new(0));
    let stub = warp::any().and(warp::body::json()).map(move |req: Value| {
        let id = id.fetch_add(1, Ordering::SeqCst);
        let ((method, params), response) = inouts[id].clone();
        assert_eq!(
            json!({"id": 1, "jsonrpc": "2.0", "method": method, "params": params}),
            req
        );
        Ok(warp::reply::json(&response))
    });
    let port = libra_config::utils::get_available_port();
    let future = warp::serve(stub).bind(([127, 0, 0, 1], port));
    tokio::task::spawn(async move { future.await });
    let server_url = format!("http://localhost:{}", port);
    Client::from_url(&server_url, Retry::default()).unwrap()
}

#[tokio::test]
async fn test_get_metadata() {
    let result = metadata_sample();
    let client = setup(("get_metadata", json!([])), new_response(result.clone()));
    let metadata = client.get_metadata().await.unwrap();
    assert_eq!(state_for_version(1), metadata.state);
    assert_eq!(result, serde_json::to_value(&*metadata).unwrap());
}

#[tokio::test]
async fn test_get_metadata_by_version() {
    let result = json!({"version": 1, "timestamp": 234234, "chain_id": 4});
    let client = setup(("get_metadata", json!([1])), new_response(result.clone()));

    let metadata = client.get_metadata_by_version(1).await.unwrap();
    assert_eq!(result, serde_json::to_value(&*metadata).unwrap());
}

#[tokio::test]
async fn test_network_error() {
    let client = Client::from_url("https://mustnotexisturl.xyz", Retry::default()).unwrap();
    assert_err!(client.get_metadata().await, Error::NetworkError{..}, true);
}

#[tokio::test]
async fn test_invalid_http_status() {
    let client = Client::from_url("https://testnet.libra.org/unknown", Retry::default()).unwrap();
    assert_err!(client.get_metadata().await, Error::InvalidHTTPStatus{..}, false);
}

#[tokio::test]
async fn test_invalid_http_response() {
    let client = setup(("get_metadata", json!([])), json!("invalid"));

    assert_err!(client.get_metadata().await, Error::InvalidHTTPResponse{..}, true);
}

#[tokio::test]
async fn test_invalid_rpc_response() {
    let client = setup(
        ("get_metadata", json!([])),
        json!({
            "id": 1,
            "jsonrpc": "1.0",
            "libra_chain_id": 4,
            "libra_ledger_timestampusec": 12112,
            "libra_ledger_version": 1
        }),
    );

    assert_err!(client.get_metadata().await, Error::InvalidRpcResponse{..}, false);
}

#[tokio::test]
async fn test_deserialize_result_error() {
    let client = setup(
        ("get_metadata", json!([])),
        json!({
            "result": {
                "version": false
            },
            "id": 1,
            "jsonrpc": "2.0",
            "libra_chain_id": 4,
            "libra_ledger_timestampusec": 12112,
            "libra_ledger_version": 1
        }),
    );

    assert_err!(client.get_metadata().await, Error::DeserializeResponseJsonError{..}, true);
}

#[tokio::test]
async fn test_jsonrpc_error() {
    let client = setup(
        ("get_metadata", json!([])),
        json!({
            "error": {
                "code": -32600, "data": null, "message": "Invalid Request",
            },
            "id": 1,
            "jsonrpc": "2.0",
            "libra_chain_id": 4,
            "libra_ledger_timestampusec": 12112,
            "libra_ledger_version": 1
        }),
    );

    assert_err!(client.get_metadata().await, Error::JsonRpcError{..}, true);
}

#[tokio::test]
async fn test_result_not_found() {
    let client = setup(
        ("get_metadata", json!([])),
        json!({
            "id": 1,
            "jsonrpc": "2.0",
            "libra_chain_id": 4,
            "libra_ledger_timestampusec": 12112,
            "libra_ledger_version": 1
        }),
    );

    assert_err!(client.get_metadata().await, Error::ResultNotFound{..}, false);
}

#[tokio::test]
async fn test_submit() {
    let result = json!(null);
    let client = setup(
        ("submit", json!([signed_txn_hex_sample()])),
        new_response(result),
    );

    let resp = client.submit(&signed_txn_sample()).await.expect("success");
    assert_eq!(state_for_version(1), resp.state);
}

#[tokio::test]
async fn test_submit_failed() {
    let client = setup(
        ("submit", json!([signed_txn_hex_sample()])),
        invalid_request_response(),
    );

    assert_err!(client.submit(&signed_txn_sample()).await, Error::JsonRpcError{..}, true);
}

#[tokio::test]
async fn test_get_account_not_found() {
    let address: AccountAddress = "d738a0b9851305dfe1d17707f0841dbc".parse().unwrap();
    let result = json!(null);
    let client = setup(("get_account", json!([address])), new_response(result));

    let ret = client.get_account(&address).await.unwrap();
    assert_eq!(state_for_version(1), ret.state);
    assert_eq!(None, *ret);
}

#[tokio::test]
async fn test_get_account() {
    let address: AccountAddress = "d738a0b9851305dfe1d17707f0841dbc".parse().unwrap();
    let result = json!({
        "address": address,
        "authentication_key": "447fc3be296803c2303951c7816624c7566730a5cc6860a4a1bd3c04731569f5",
        "balances": [],
        "delegated_key_rotation_capability": false,
        "delegated_withdrawal_capability": false,
        "is_frozen": false,
        "received_events_key": "02000000000000000000000000000000000000000a550c18",
        "role": { "type": "unknown" },
        "sent_events_key": "03000000000000000000000000000000000000000a550c18",
        "sequence_number": 1
    });
    let client = setup(("get_account", json!([address])), new_response(result));

    let ret = client
        .get_account(&address)
        .await
        .expect("ok")
        .result
        .expect("some");
    assert_eq!(
        json!({
            "address": address,
            "authentication_key": "447fc3be296803c2303951c7816624c7566730a5cc6860a4a1bd3c04731569f5",
            "received_events_key": "02000000000000000000000000000000000000000a550c18",
            "role": { "type": "unknown" },
            "sent_events_key": "03000000000000000000000000000000000000000a550c18",
            "sequence_number": 1
        }),
        serde_json::to_value(&ret).unwrap()
    );
    assert!(ret.balances.is_empty());
    assert!(!ret.delegated_key_rotation_capability);
    assert!(!ret.delegated_withdrawal_capability);
    assert!(!ret.is_frozen);
}

#[tokio::test]
async fn test_get_account_transaction_not_found() {
    let address: AccountAddress = "d738a0b9851305dfe1d17707f0841dbc".parse().unwrap();
    let result = json!(null);
    let client = setup(
        ("get_account_transaction", json!([address, 1, false])),
        new_response(result),
    );

    let ret = client
        .get_account_transaction(&address, 1, false)
        .await
        .unwrap();
    assert_eq!(None, *ret);
}

#[tokio::test]
async fn test_get_account_transaction() {
    let address: AccountAddress = "d738a0b9851305dfe1d17707f0841dbc".parse().unwrap();
    let result = transaction_sample();
    let client = setup(
        ("get_account_transaction", json!([address, 1, true])),
        new_response(result.clone()),
    );

    let ret = client
        .get_account_transaction(&address, 1, true)
        .await
        .unwrap();

    assert_eq!(result, to_value(&*ret).unwrap());
}

#[tokio::test]
async fn test_get_account_transactions() {
    let address: AccountAddress = "d738a0b9851305dfe1d17707f0841dbc".parse().unwrap();
    let result = json!([transaction_sample()]);
    let client = setup(
        ("get_account_transactions", json!([address, 0, 1, true])),
        new_response(result.clone()),
    );

    let ret = client
        .get_account_transactions(&address, 0, 1, true)
        .await
        .unwrap();

    assert_eq!(result, to_value(&*ret).unwrap());
}

#[tokio::test]
async fn test_get_account_transactions_not_found() {
    let address: AccountAddress = "d738a0b9851305dfe1d17707f0841dbc".parse().unwrap();
    let result = json!([]);
    let client = setup(
        ("get_account_transactions", json!([address, 0, 1, true])),
        new_response(result.clone()),
    );

    let ret = client
        .get_account_transactions(&address, 0, 1, true)
        .await
        .unwrap();

    assert_eq!(result, to_value(&*ret).unwrap());
}

#[tokio::test]
async fn test_get_transactions() {
    let result = json!([transaction_sample()]);
    let client = setup(
        ("get_transactions", json!([0, 1, true])),
        new_response(result.clone()),
    );

    let ret = client.get_transactions(0, 1, true).await.unwrap();

    assert_eq!(result, to_value(&*ret).unwrap());
}

#[tokio::test]
async fn test_get_events() {
    let result = events_sample();
    let client = setup(
        ("get_events", json!(["key", 0, 1])),
        new_response(result.clone()),
    );

    let ret = client.get_events("key", 0, 1).await.unwrap();

    assert_eq!(result, to_value(&*ret).unwrap());
}

#[tokio::test]
async fn test_get_currencies() {
    let result = currencies_sample();
    let client = setup(("get_currencies", json!([])), new_response(result.clone()));

    let ret = client.get_currencies().await.unwrap();

    assert_eq!(result, to_value(&*ret).unwrap());
}

#[tokio::test]
async fn test_get_state_proof() {
    let result = json!({
        "epoch_change_proof": "epoch_change_proof",
        "ledger_info_with_signatures": "ledger_info_with_signatures",
        "ledger_consistency_proof": "ledger_consistency_proof"
    });
    let client = setup(
        ("get_state_proof", json!([1])),
        new_response(result.clone()),
    );

    let ret = client.get_state_proof(1).await.unwrap();

    assert_eq!(result, to_value(&*ret).unwrap());
}

#[tokio::test]
async fn test_get_account_state_with_proof() {
    let address: AccountAddress = "d738a0b9851305dfe1d17707f0841dbc".parse().unwrap();
    let from_version = 1;
    let to_version = 2;
    let result = json!({
        "version": 1,
        "blob": "blob",
        "proof": {
            "ledger_info_to_transaction_info_proof": "ledger_info_to_transaction_info_proof",
            "transaction_info": "transaction_info",
            "transaction_info_to_account_proof": "transaction_info_to_account_proof"
        }
    });
    let client = setup(
        (
            "get_account_state_with_proof",
            json!([address, from_version, to_version]),
        ),
        new_response(result.clone()),
    );

    let ret = client
        .get_account_state_with_proof(&address, Some(from_version), Some(to_version))
        .await
        .unwrap();

    assert_eq!(result, to_value(&*ret).unwrap());
}

#[tokio::test]
async fn test_update_last_known_state() {
    let client = setup_batch(vec![
        (
            ("get_metadata", json!([])),
            new_response_with_version(metadata_sample(), 1),
        ),
        (
            ("get_metadata", json!([])),
            new_response_with_version(metadata_sample(), 2),
        ),
    ]);

    assert_eq!(client.last_known_state(), None);

    client.get_metadata().await.expect("some");
    client.get_metadata().await.expect("some");

    assert_eq!(client.last_known_state().unwrap(), state_for_version(2))
}

#[tokio::test]
async fn test_submit_method_returns_stale_response() {
    let client = setup_batch(vec![
        (
            ("get_metadata", json!([])),
            new_response_with_version(metadata_sample(), 2),
        ),
        (
            ("submit", json!([signed_txn_hex_sample()])),
            new_response_with_version(json!(null), 1),
        ),
    ]);

    client.get_metadata().await.expect("some");
    let ret = client.submit(&signed_txn_sample()).await;
    assert_err!(ret, Error::StaleResponseError{..}, false);
}

#[tokio::test]
async fn test_retry_stale_response_on_get_methods() {
    let client = setup_batch(vec![
        (
            ("get_metadata", json!([])),
            new_response_with_version(metadata_sample(), 2),
        ),
        (
            ("get_metadata", json!([])),
            new_response_with_version(metadata_sample(), 1),
        ),
        (
            ("get_metadata", json!([])),
            new_response_with_version(metadata_sample(), 3),
        ),
    ]);

    client.get_metadata().await.expect("some");
    let now = std::time::Instant::now();
    client.get_metadata().await.expect("some");
    assert!(now.elapsed() > Duration::from_millis(10));

    assert_eq!(client.last_known_state().unwrap(), state_for_version(3))
}

#[tokio::test]
async fn test_wait_for_transaction() {
    let address: AccountAddress = "d738a0b9851305dfe1d17707f0841dbc".parse().unwrap();
    let seq = 3;
    let client = setup_batch(vec![
        (
            ("get_account_transaction", json!([address, seq, true])),
            new_response_with_version(json!(null), 2),
        ),
        (
            ("get_account_transaction", json!([address, seq, true])),
            new_response_with_version(json!(null), 3),
        ),
        (
            ("get_account_transaction", json!([address, seq, true])),
            new_response_with_version(json!(null), 1),
        ),
        (
            ("get_account_transaction", json!([address, seq, true])),
            new_response_with_version(json!(transaction_sample()), 4),
        ),
    ]);

    let expiration_time_secs = (version_timestamp(10) / 1_000_000) as u64;
    let txn_hash = "0895ced78e9f10c1351f248b9bda40cc9c8e6d4dbba823e08fe1cfacd582f171";
    client
        .wait_for_transaction(&address, seq, expiration_time_secs, txn_hash, None, None)
        .await
        .expect("some");
}

#[tokio::test]
async fn test_wait_for_signed_transaction() {
    let txn = signed_txn_sample();
    let address = txn.sender();
    let seq = txn.sequence_number();

    let client = setup(
        ("get_account_transaction", json!([address, seq, true])),
        new_response_with_version(json!(transaction_sample()), 4),
    );

    client
        .wait_for_signed_transaction(&txn, None, None)
        .await
        .expect("some");
}

#[tokio::test]
async fn test_wait_for_transaction_error_get_transaction_error() {
    let address: AccountAddress = "d738a0b9851305dfe1d17707f0841dbc".parse().unwrap();
    let seq = 3;
    let client = setup(
        ("get_account_transaction", json!([address, seq, true])),
        invalid_request_response(),
    );

    let expiration_time_secs = (version_timestamp(10) / 1_000_000) as u64;
    let txn_hash = "mismatched hash";
    let err = client
        .wait_for_transaction(&address, seq, expiration_time_secs, txn_hash, None, None)
        .await;

    assert_err!(err, WaitForTransactionError::GetTransactionError{..}, true);
}

#[tokio::test]
async fn test_wait_for_transaction_error_hash_mismatch() {
    let address: AccountAddress = "d738a0b9851305dfe1d17707f0841dbc".parse().unwrap();
    let seq = 3;
    let client = setup(
        ("get_account_transaction", json!([address, seq, true])),
        new_response_with_version(json!(transaction_sample()), 4),
    );

    let expiration_time_secs = (version_timestamp(10) / 1_000_000) as u64;
    let txn_hash = "mismatched hash";
    let err = client
        .wait_for_transaction(&address, seq, expiration_time_secs, txn_hash, None, None)
        .await;

    assert_err!(err, WaitForTransactionError::TransactionHashMismatchError {..}, false);
}

#[tokio::test]
async fn test_wait_for_transaction_error_execution_failed() {
    let address: AccountAddress = "d738a0b9851305dfe1d17707f0841dbc".parse().unwrap();
    let seq = 3;
    let client = setup(
        ("get_account_transaction", json!([address, seq, true])),
        new_response_with_version(json!(custom_transaction_sample("out_of_gas", json!([]))), 4),
    );

    let expiration_time_secs = (version_timestamp(10) / 1_000_000) as u64;
    let txn_hash = "0895ced78e9f10c1351f248b9bda40cc9c8e6d4dbba823e08fe1cfacd582f171";
    let err = client
        .wait_for_transaction(&address, seq, expiration_time_secs, txn_hash, None, None)
        .await;

    assert_err!(
        err,
        WaitForTransactionError::TransactionExecutionFailed { .. },
        false
    );
}

#[tokio::test]
async fn test_wait_for_transaction_error_txn_expired() {
    let address: AccountAddress = "d738a0b9851305dfe1d17707f0841dbc".parse().unwrap();
    let seq = 3;
    let client = setup(
        ("get_account_transaction", json!([address, seq, true])),
        new_response_with_version(json!(null), 2),
    );

    // expiration timestamp is from version 1, older than respond timestamp for version 2
    let expiration_time_secs = (version_timestamp(1) / 1_000_000) as u64;
    let txn_hash = "0895ced78e9f10c1351f248b9bda40cc9c8e6d4dbba823e08fe1cfacd582f171";
    let err = client
        .wait_for_transaction(&address, seq, expiration_time_secs, txn_hash, None, None)
        .await;
    assert_err!(
        err,
        WaitForTransactionError::TransactionExpired { .. },
        false
    );
}

#[tokio::test]
async fn test_wait_for_transaction_error_timeout() {
    let address: AccountAddress = "d738a0b9851305dfe1d17707f0841dbc".parse().unwrap();
    let seq = 3;
    let client = setup(
        ("get_account_transaction", json!([address, seq, true])),
        new_response_with_version(json!(null), 2),
    );

    // expiration timestamp is from version 1, older than respond timestamp for version 2
    let expiration_time_secs = (version_timestamp(3) / 1_000_000) as u64;
    let txn_hash = "0895ced78e9f10c1351f248b9bda40cc9c8e6d4dbba823e08fe1cfacd582f171";
    let err = client
        .wait_for_transaction(
            &address,
            seq,
            expiration_time_secs,
            txn_hash,
            Some(Duration::from_millis(10)), // timeout
            Some(Duration::from_millis(20)), // delay
        )
        .await;
    assert_err!(
        err,
        WaitForTransactionError::Timeout { .. },
        false
    );
}

#[tokio::test]
async fn test_chain_id_mismatch_error() {
    let client = setup_batch(vec![
        (("get_metadata", json!([])), new_response(metadata_sample())),
        (
            ("get_metadata", json!([])),
            json!({
                    "error": {
                        "code": -32600, "data": null, "message": "Invalid Request",
                    },
                    "id": 1,
                    "jsonrpc": "2.0",
                    "libra_chain_id": 9,
                    "libra_ledger_timestampusec": 12112,
                    "libra_ledger_version": 1
            }),
        ),
    ]);

    client.get_metadata().await.expect("some");
    let err = client.get_metadata().await;
    assert_err!(err, Error::ChainIdMismatch { .. }, false);
}

fn currencies_sample() -> Value {
    json!([
        {
            "burn_events_key": "06000000000000000000000000000000000000000a550c18",
            "cancel_burn_events_key": "08000000000000000000000000000000000000000a550c18",
            "code": "Coin1",
            "exchange_rate_update_events_key": "09000000000000000000000000000000000000000a550c18",
            "fractional_part": 100,
            "mint_events_key": "05000000000000000000000000000000000000000a550c18",
            "preburn_events_key": "07000000000000000000000000000000000000000a550c18",
            "scaling_factor": 1000000,
            "to_lbr_exchange_rate": 1.0,
        },
        {
            "burn_events_key": "0b000000000000000000000000000000000000000a550c18",
            "cancel_burn_events_key": "0d000000000000000000000000000000000000000a550c18",
            "code": "LBR",
            "exchange_rate_update_events_key": "0e000000000000000000000000000000000000000a550c18",
            "fractional_part": 1000,
            "mint_events_key": "0a000000000000000000000000000000000000000a550c18",
            "preburn_events_key": "0c000000000000000000000000000000000000000a550c18",
            "scaling_factor": 1000000,
            "to_lbr_exchange_rate": 1.0
        }
    ])
}

fn events_sample() -> Value {
    json!([
        {
          "data": {
            "amount": {
              "amount": 3000000,
              "currency": "LBR"
            },
            "metadata": "metadata",
            "receiver": "762cbea8b99911d49707d2b901e13425",
            "sender": "876ff1d441cb0b352f438fcfbce8608f",
            "type": "sentpayment"
          },
          "key": "0300000000000000876ff1d441cb0b352f438fcfbce8608f",
          "sequence_number": 1,
          "transaction_version": 27
        },
        {
          "data": {
            "amount": {
              "amount": 3000000,
              "currency": "LBR"
            },
            "metadata": "metadata",
            "receiver": "762cbea8b99911d49707d2b901e13425",
            "sender": "876ff1d441cb0b352f438fcfbce8608f",
            "type": "receivedpayment"
          },
          "key": "0000000000000000762cbea8b99911d49707d2b901e13425",
          "sequence_number": 1,
          "transaction_version": 27
        }
    ])
}

fn transaction_sample() -> Value {
    custom_transaction_sample("executed", events_sample())
}

fn custom_transaction_sample(vm_status_type: &str, events: Value) -> Value {
    json!({
      "bytes": format!("00{}", signed_txn_hex_sample()),
      "events": events,
      "gas_used": 476,
      "hash": "0895ced78e9f10c1351f248b9bda40cc9c8e6d4dbba823e08fe1cfacd582f171",
      "transaction": {
        "chain_id": 4,
        "expiration_timestamp_secs": 1600152999,
        "gas_currency": "LBR",
        "gas_unit_price": 10,
        "max_gas_amount": 1000000,
        "public_key": "f4d854d7719c8ca6b454df278aa513c3e1f17ad6a38579ff0baa2bd50f5dd0e5",
        "script": {
          "amount": 3000000,
          "currency": "LBR",
          "metadata": "metadata",
          "metadata_signature": "metadata_signature",
          "receiver": "762cbea8b99911d49707d2b901e13425",
          "type": "peer_to_peer_transaction"
        },
        "script_bytes": "e101a11ceb0b010000000701000202020403061004160205181d0735610896011000000001010000020001000003020301010004010300010501060c0108000506080005030a020a020005060c05030a020a020109000c4c696272614163636f756e741257697468647261774361706162696c6974791b657874726163745f77697468647261775f6361706162696c697479087061795f66726f6d1b726573746f72655f77697468647261775f6361706162696c69747900000000000000000000000000000001010104010c0b0011000c050e050a010a020b030b0438000b05110202010700000000000000000000000000000001034c4252034c4252000403762cbea8b99911d49707d2b901e1342501c0c62d000000000004000400",
        "script_hash": "61749d43d8f10940be6944df85ddf13f0f8fb830269c601f481cc5ee3de731c8",
        "sender": "876ff1d441cb0b352f438fcfbce8608f",
        "sequence_number": 1,
        "signature": "79358f2a818adcbfb7e9db3e9a560007881acfe6b62c29f1eedcc76b534024a574fcb95bf4c438da33eba8052b565c8d7e3f507225d2998849d6613f64fa6005",
        "signature_scheme": "Scheme::Ed25519",
        "type": "user"
      },
      "version": 27,
      "vm_status": {
        "type": vm_status_type
      }
    })
}

fn metadata_sample() -> Value {
    json!({"version": 1, "timestamp": 234234, "chain_id": 4})
}

fn signed_txn_sample() -> SignedTransaction {
    let bytes = hex::decode(signed_txn_hex_sample()).unwrap();
    lcs::from_bytes(&bytes).unwrap()
}

fn signed_txn_hex_sample() -> String {
    "876ff1d441cb0b352f438fcfbce8608f010000000000000001e101a11ceb0b010000000701000202020403061004160205181d0735610896011000000001010000020001000003020301010004010300010501060c0108000506080005030a020a020005060c05030a020a020109000c4c696272614163636f756e741257697468647261774361706162696c6974791b657874726163745f77697468647261775f6361706162696c697479087061795f66726f6d1b726573746f72655f77697468647261775f6361706162696c69747900000000000000000000000000000001010104010c0b0011000c050e050a010a020b030b0438000b05110202010700000000000000000000000000000001034c4252034c4252000403762cbea8b99911d49707d2b901e1342501c0c62d00000000000400040040420f00000000000000000000000000034c4252a765605f00000000040020f4d854d7719c8ca6b454df278aa513c3e1f17ad6a38579ff0baa2bd50f5dd0e54079358f2a818adcbfb7e9db3e9a560007881acfe6b62c29f1eedcc76b534024a574fcb95bf4c438da33eba8052b565c8d7e3f507225d2998849d6613f64fa6005".to_string()
}

fn invalid_request_response() -> Value {
    json!({
            "error": {
                "code": -32600, "data": null, "message": "Invalid Request",
            },
            "id": 1,
            "jsonrpc": "2.0",
            "libra_chain_id": 4,
            "libra_ledger_timestampusec": 12112,
            "libra_ledger_version": 1
    })
}

fn new_response(result: Value) -> Value {
    new_response_with_version(result, 1)
}

fn new_response_with_version(result: Value, version: u64) -> Value {
    to_value(JsonRpcResponse {
        libra_chain_id: 4,
        libra_ledger_version: version,
        libra_ledger_timestampusec: version_timestamp(version),
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        result: Some(result),
        error: None,
    })
    .unwrap()
}

fn state_for_version(version: u64) -> State {
    State {
        chain_id: 4,
        version,
        timestamp_usecs: version_timestamp(version),
    }
}

fn version_timestamp(version: u64) -> u64 {
    1602888395 * 1000000 + version * 1000000
}

fn assert_error_source(err: &dyn std::error::Error, has_source: bool)
where
    Error: std::error::Error,
{
    assert_eq!(
        has_source,
        err.source().is_some(),
        "expected error source, but it is None: {}",
        err
    );
}
