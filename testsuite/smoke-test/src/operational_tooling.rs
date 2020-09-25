// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    smoke_test_environment::SmokeTestEnvironment,
    test_utils::{
        compare_balances, load_backend_storage, load_libra_root_storage,
        setup_swarm_and_client_proxy,
    },
};
use libra_config::config::NodeConfig;
use libra_global_constants::{
    CONSENSUS_KEY, OPERATOR_ACCOUNT, OPERATOR_KEY, VALIDATOR_NETWORK_KEY,
};
use libra_management::storage::to_x25519;
use libra_operational_tool::test_helper::OperationalTool;
use libra_secure_json_rpc::VMStatusView;
use libra_secure_storage::{CryptoStorage, KVStorage, Storage};
use libra_types::{
    account_address::AccountAddress,
    account_config::{testnet_dd_account_address, treasury_compliance_account_address},
    chain_id::ChainId,
    transaction::authenticator::AuthenticationKey,
};
use std::convert::{TryFrom, TryInto};

#[test]
fn test_consensus_key_rotation() {
    let mut swarm = SmokeTestEnvironment::new(5);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&&node_config);

    // Rotate the consensus key
    let (txn_ctx, new_consensus_key) = op_tool.rotate_consensus_key(&backend).unwrap();
    let mut client = swarm.get_validator_client(0, None);
    client
        .wait_for_transaction(txn_ctx.address, txn_ctx.sequence_number + 1)
        .unwrap();

    // Verify that the config has been updated correctly with the new consensus key
    let validator_account = node_config.validator_network.as_ref().unwrap().peer_id();
    let config_consensus_key = op_tool
        .validator_config(validator_account, &backend)
        .unwrap()
        .consensus_public_key;
    assert_eq!(new_consensus_key, config_consensus_key);

    // Verify that the validator set info contains the new consensus key
    let info_consensus_key = op_tool.validator_set(validator_account, &backend).unwrap()[0]
        .consensus_public_key
        .clone();
    assert_eq!(new_consensus_key, info_consensus_key);

    // Rotate the consensus key in storage manually and perform another rotation using the op_tool.
    // Here, we expected the op_tool to see that the consensus key in storage doesn't match the one
    // on-chain, and thus it should simply forward a transaction to the blockchain.
    let mut storage: Storage = (&backend).try_into().unwrap();
    let rotated_consensus_key = storage.rotate_key(CONSENSUS_KEY).unwrap();
    let (_txn_ctx, new_consensus_key) = op_tool.rotate_consensus_key(&backend).unwrap();
    assert_eq!(rotated_consensus_key, new_consensus_key);
}

#[test]
fn test_operator_key_rotation() {
    let mut swarm = SmokeTestEnvironment::new(5);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = OperationalTool::new(
        format!("http://127.0.0.1:{}", node_config.rpc.address.port()),
        ChainId::test(),
    );

    // Load validator's on disk storage
    let backend = load_backend_storage(&&node_config);

    let (txn_ctx, _) = op_tool.rotate_operator_key(&backend).unwrap();
    let mut client = swarm.get_validator_client(0, None);
    client
        .wait_for_transaction(txn_ctx.address, txn_ctx.sequence_number + 1)
        .unwrap();

    // Verify that the transaction was executed correctly
    let result = op_tool
        .validate_transaction(txn_ctx.address, txn_ctx.sequence_number)
        .unwrap();
    let vm_status = result.unwrap();
    assert_eq!(VMStatusView::Executed, vm_status);

    // Rotate the consensus key to verify the operator key has been updated
    let (txn_ctx, new_consensus_key) = op_tool.rotate_consensus_key(&backend).unwrap();
    let mut client = swarm.get_validator_client(0, None);
    client
        .wait_for_transaction(txn_ctx.address, txn_ctx.sequence_number + 1)
        .unwrap();

    // Verify that the config has been updated correctly with the new consensus key
    let validator_account = node_config.validator_network.as_ref().unwrap().peer_id();
    let config_consensus_key = op_tool
        .validator_config(validator_account, &backend)
        .unwrap()
        .consensus_public_key;
    assert_eq!(new_consensus_key, config_consensus_key);
}

#[test]
fn test_operator_key_rotation_recovery() {
    let mut swarm = SmokeTestEnvironment::new(5);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = OperationalTool::new(
        format!("http://127.0.0.1:{}", node_config.rpc.address.port()),
        ChainId::test(),
    );

    // Load validator's on disk storage
    let backend = load_backend_storage(&&node_config);

    // Rotate the operator key
    let (txn_ctx, new_operator_key) = op_tool.rotate_operator_key(&backend).unwrap();
    let mut client = swarm.get_validator_client(0, None);
    client
        .wait_for_transaction(txn_ctx.address, txn_ctx.sequence_number + 1)
        .unwrap();

    // Verify that the transaction was executed correctly
    let result = op_tool
        .validate_transaction(txn_ctx.address, txn_ctx.sequence_number)
        .unwrap();
    let vm_status = result.unwrap();
    assert_eq!(VMStatusView::Executed, vm_status);

    // Verify that the operator key was updated on-chain
    let mut storage: Storage = (&backend).try_into().unwrap();
    let operator_account = storage
        .get::<AccountAddress>(OPERATOR_ACCOUNT)
        .unwrap()
        .value;
    let account_resource = op_tool.account_resource(operator_account).unwrap();
    let on_chain_operator_key = hex::decode(account_resource.authentication_key).unwrap();
    assert_eq!(
        AuthenticationKey::ed25519(&new_operator_key),
        AuthenticationKey::try_from(on_chain_operator_key).unwrap()
    );

    // Rotate the operator key in storage manually and perform another rotation using the op tool.
    // Here, we expected the op_tool to see that the operator key in storage doesn't match the one
    // on-chain, and thus it should simply forward a transaction to the blockchain.
    let rotated_operator_key = storage.rotate_key(OPERATOR_KEY).unwrap();
    let (txn_ctx, new_operator_key) = op_tool.rotate_operator_key(&backend).unwrap();
    assert_eq!(rotated_operator_key, new_operator_key);
    client
        .wait_for_transaction(txn_ctx.address, txn_ctx.sequence_number + 1)
        .unwrap();

    // Verify that the transaction was executed correctly
    let result = op_tool
        .validate_transaction(txn_ctx.address, txn_ctx.sequence_number)
        .unwrap();
    let vm_status = result.unwrap();
    assert_eq!(VMStatusView::Executed, vm_status);

    // Verify that the operator key was updated on-chain
    let account_resource = op_tool.account_resource(operator_account).unwrap();
    let on_chain_operator_key = hex::decode(account_resource.authentication_key).unwrap();
    assert_eq!(
        AuthenticationKey::ed25519(&new_operator_key),
        AuthenticationKey::try_from(on_chain_operator_key).unwrap()
    );
}

#[test]
fn test_network_key_rotation() {
    let num_nodes = 5;
    let mut swarm = SmokeTestEnvironment::new(num_nodes);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&&node_config);

    // Rotate the validator network key
    let (txn_ctx, new_network_key) = op_tool.rotate_validator_network_key(&backend).unwrap();
    wait_for_transaction_on_all_nodes(
        &swarm,
        num_nodes,
        txn_ctx.address,
        txn_ctx.sequence_number + 1,
    );

    // Verify that config has been loaded correctly with new key
    let validator_account = node_config.validator_network.as_ref().unwrap().peer_id();
    let config_network_key = op_tool
        .validator_config(validator_account, &backend)
        .unwrap()
        .validator_network_address
        .find_noise_proto()
        .unwrap();
    assert_eq!(new_network_key, config_network_key);

    // Verify that the validator set info contains the new network key
    let info_network_key = op_tool.validator_set(validator_account, &backend).unwrap()[0]
        .validator_network_address
        .find_noise_proto()
        .unwrap();
    assert_eq!(new_network_key, info_network_key);

    // Restart validator
    // At this point, the `add_node` call ensures connectivity to all nodes
    swarm.validator_swarm.kill_node(0);
    swarm.validator_swarm.add_node(0).unwrap();
}

#[test]
fn test_network_key_rotation_recovery() {
    let num_nodes = 5;
    let mut swarm = SmokeTestEnvironment::new(num_nodes);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&&node_config);

    // Rotate the network key in storage manually and perform a key rotation using the op_tool.
    // Here, we expected the op_tool to see that the network key in storage doesn't match the one
    // on-chain, and thus it should simply forward a transaction to the blockchain.
    let mut storage: Storage = (&backend).try_into().unwrap();
    let rotated_network_key = storage.rotate_key(VALIDATOR_NETWORK_KEY).unwrap();
    let (txn_ctx, new_network_key) = op_tool.rotate_validator_network_key(&backend).unwrap();
    assert_eq!(new_network_key, to_x25519(rotated_network_key).unwrap());

    // Ensure all nodes have received the transaction
    wait_for_transaction_on_all_nodes(
        &swarm,
        num_nodes,
        txn_ctx.address,
        txn_ctx.sequence_number + 1,
    );

    // Verify that config has been loaded correctly with new key
    let validator_account = node_config.validator_network.as_ref().unwrap().peer_id();
    let config_network_key = op_tool
        .validator_config(validator_account, &backend)
        .unwrap()
        .validator_network_address
        .find_noise_proto()
        .unwrap();
    assert_eq!(new_network_key, config_network_key);

    // Verify that the validator set info contains the new network key
    let info_network_key = op_tool.validator_set(validator_account, &backend).unwrap()[0]
        .validator_network_address
        .find_noise_proto()
        .unwrap();
    assert_eq!(new_network_key, info_network_key);

    // Restart validator
    // At this point, the `add_node` call ensures connectivity to all nodes
    swarm.validator_swarm.kill_node(0);
    swarm.validator_swarm.add_node(0).unwrap();
}

#[test]
fn test_e2e_reconfiguration() {
    let (env, mut client_proxy_1) = setup_swarm_and_client_proxy(3, 1);
    let node_configs: Vec<_> = env
        .validator_swarm
        .config
        .config_files
        .iter()
        .map(|config_path| NodeConfig::load(config_path).unwrap())
        .collect();

    // the client connected to the removed validator
    let mut client_proxy_0 = env.get_validator_client(0, None);
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_0.set_accounts(client_proxy_1.copy_all_accounts());
    client_proxy_1
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
        .unwrap();
    client_proxy_1
        .wait_for_transaction(treasury_compliance_account_address(), 1)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "Coin1".to_string())],
        client_proxy_1.get_balances(&["b", "0"]).unwrap(),
    ));
    // wait for the mint txn in node 0
    client_proxy_0
        .wait_for_transaction(testnet_dd_account_address(), 1)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap(),
    ));
    let peer_id = env.get_validator(0).unwrap().validator_peer_id().unwrap();
    let op_tool = env.get_op_tool(1);
    let libra_root = load_libra_root_storage(node_configs.first().unwrap());
    let context = op_tool.remove_validator(peer_id, &libra_root).unwrap();
    client_proxy_1
        .wait_for_transaction(context.address, context.sequence_number + 1)
        .unwrap();
    // mint another 10 coins after remove node 0
    client_proxy_1
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy_1.get_balances(&["b", "0"]).unwrap(),
    ));
    // client connected to removed validator can not see the update
    assert!(compare_balances(
        vec![(10.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap(),
    ));
    // Add the node back
    let context = op_tool.add_validator(peer_id, &libra_root).unwrap();
    client_proxy_0
        .wait_for_transaction(context.address, context.sequence_number)
        .unwrap();
    // Wait for it catches up, mint1 + mint2 => seq == 2
    client_proxy_0
        .wait_for_transaction(testnet_dd_account_address(), 2)
        .unwrap();
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap(),
    ));
}

fn wait_for_transaction_on_all_nodes(
    swarm: &SmokeTestEnvironment,
    num_nodes: usize,
    account: AccountAddress,
    sequence_number: u64,
) {
    for i in 0..num_nodes {
        let mut client = swarm.get_validator_client(i, None);
        client
            .wait_for_transaction(account, sequence_number)
            .unwrap();
    }
}
