// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    operational_tooling::{
        launch_swarm_with_op_tool_and_backend, wait_for_transaction_on_all_nodes,
    },
    smoke_test_environment::new_local_swarm,
    test_utils::diem_swarm_utils::load_validators_backend_storage,
};
use diem_config::config::SecureBackend;
use diem_global_constants::OWNER_ACCOUNT;
use diem_operational_tool::test_helper::OperationalTool;
use diem_sdk::client::views::VMStatusView;
use diem_secure_storage::{KVStorage, Storage};
use diem_types::{account_address::AccountAddress, network_address::NetworkAddress};
use forge::{LocalSwarm, NodeExt};
use std::{convert::TryInto, str::FromStr};

#[test]
fn test_consensus_observer_mode_storage_error() {
    let num_nodes = 4;
    let (swarm, op_tool, backend, _) = launch_swarm_with_op_tool_and_backend(num_nodes);

    // Kill safety rules storage for validator 1 to ensure it fails on the next epoch change
    let node_config = swarm.validators().nth(1).unwrap().config().clone();
    let safety_rules_storage = match node_config.consensus.safety_rules.backend {
        SecureBackend::OnDiskStorage(config) => SecureBackend::OnDiskStorage(config),
        _ => panic!("On-disk storage is the only backend supported in smoke tests"),
    };
    let mut safety_rules_storage: Storage = (&safety_rules_storage).try_into().unwrap();
    safety_rules_storage.reset_and_clear().unwrap();

    // Force a new epoch by updating validator 0's full node address in the validator config
    let txn_ctx = op_tool
        .set_validator_config(
            None,
            Some(NetworkAddress::from_str("/ip4/10.0.0.16/tcp/80").unwrap()),
            &backend,
            false,
            false,
        )
        .unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Rotate validator 0's operator key several different times, each requiring a new transaction
    for _ in 0..5 {
        let (txn_ctx, _) = op_tool.rotate_operator_key(&backend, false).unwrap();
        assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());
    }

    // Verify validator 1 is still able to stay up to date with validator 0 (despite safety rules failing)
    let client_0 = swarm.validators().next().unwrap().json_rpc_client();
    let sequence_number_0 = client_0
        .get_account(txn_ctx.address)
        .unwrap()
        .into_inner()
        .unwrap()
        .sequence_number;
    let client_1 = swarm.validators().nth(1).unwrap().json_rpc_client();
    let sequence_number_1 = client_1
        .get_account(txn_ctx.address)
        .unwrap()
        .into_inner()
        .unwrap()
        .sequence_number;
    assert_eq!(sequence_number_0, sequence_number_1);
}

#[test]
fn test_safety_rules_export_consensus() {
    // Create the smoke test environment
    let num_nodes = 4;
    let mut swarm = new_local_swarm(num_nodes);

    // Update all nodes to export the consensus key
    for validator in swarm.validators_mut() {
        let mut node_config = validator.config().clone();
        node_config.consensus.safety_rules.export_consensus_key = true;
        node_config.save(validator.config_path()).unwrap();
        validator.restart().unwrap();
    }

    // Launch and test the swarm
    swarm.launch().unwrap();
    rotate_operator_and_consensus_key(swarm);
}

#[test]
fn test_safety_rules_export_consensus_compatibility() {
    // Create the smoke test environment
    let num_nodes = 4;
    let mut swarm = new_local_swarm(num_nodes);

    // Allow the first and second nodes to export the consensus key
    for validator in swarm.validators_mut().take(2) {
        let mut node_config = validator.config().clone();
        node_config.consensus.safety_rules.export_consensus_key = true;
        node_config.save(validator.config_path()).unwrap();
        validator.restart().unwrap();
    }

    // Launch and test the swarm
    swarm.launch().unwrap();
    rotate_operator_and_consensus_key(swarm);
}

fn rotate_operator_and_consensus_key(swarm: LocalSwarm) {
    use forge::Node;

    let validator = swarm.validators().next().unwrap();
    let json_rpc_endpoint = validator.json_rpc_endpoint().to_string();

    // Load the first validator's on disk storage
    let backend = load_validators_backend_storage(validator);
    let storage: Storage = (&backend).try_into().unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = OperationalTool::new(json_rpc_endpoint, swarm.chain_id());

    // Rotate the first node's operator key
    let (txn_ctx, _) = op_tool.rotate_operator_key(&backend, true).unwrap();
    assert!(txn_ctx.execution_result.is_none());

    // Ensure all nodes have received the transaction
    wait_for_transaction_on_all_nodes(&swarm, txn_ctx.address, txn_ctx.sequence_number);

    // Rotate the consensus key to verify the operator key has been updated
    let (txn_ctx, new_consensus_key) = op_tool.rotate_consensus_key(&backend, false).unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Ensure all nodes have received the transaction
    wait_for_transaction_on_all_nodes(&swarm, txn_ctx.address, txn_ctx.sequence_number);

    // Verify that the config has been updated correctly with the new consensus key
    let validator_account = storage.get::<AccountAddress>(OWNER_ACCOUNT).unwrap().value;
    let config_consensus_key = op_tool
        .validator_config(validator_account, Some(&backend))
        .unwrap()
        .consensus_public_key;
    assert_eq!(new_consensus_key, config_consensus_key);
}
