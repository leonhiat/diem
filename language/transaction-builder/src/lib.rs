// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_types::{
    account_address::AccountAddress,
    block_metadata::BlockMetadata,
    transaction::{authenticator::AuthenticationKey, Script, Transaction, TransactionArgument},
};
use std::convert::TryFrom;
use stdlib::transaction_scripts::StdlibScript;
#[cfg(any(test, feature = "fuzzing"))]
use vm::file_format::{Bytecode, CompiledScript};

fn validate_auth_key_prefix(auth_key_prefix: &[u8]) {
    let auth_key_prefix_length = auth_key_prefix.len();
    assert!(
        auth_key_prefix_length == 0
            || auth_key_prefix_length == AuthenticationKey::LENGTH - AccountAddress::LENGTH,
        "Bad auth key prefix length {}",
        auth_key_prefix_length
    );
}

/// Encode a program adding `new_validator` to the pending validator set. Fails if the
/// `new_validator` address is already in the validator set, already in the pending valdiator set,
/// or does not have a `ValidatorConfig` resource stored at the address
pub fn encode_add_validator_script(new_validator: &AccountAddress) -> Script {
    Script::new(
        StdlibScript::AddValidator.compiled_bytes().into_vec(),
        vec![TransactionArgument::Address(*new_validator)],
    )
}

/// Encode a program transferring `amount` coins from `sender` to `recipient`. Fails if there is no
/// account at the recipient address or if the sender's balance is lower than `amount`.
pub fn encode_transfer_script(
    recipient: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    amount: u64,
) -> Script {
    validate_auth_key_prefix(&auth_key_prefix);
    Script::new(
        StdlibScript::PeerToPeer.compiled_bytes().into_vec(),
        vec![
            TransactionArgument::Address(*recipient),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U64(amount),
        ],
    )
}

/// Encode a program transferring `amount` coins from `sender` to `recipient` with associated
/// metadata `metadata`. Fails if there is no account at the recipient address or if the sender's
/// balance is lower than `amount`.
pub fn encode_transfer_with_metadata_script(
    recipient: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    amount: u64,
    metadata: Vec<u8>,
) -> Script {
    validate_auth_key_prefix(&auth_key_prefix);
    Script::new(
        StdlibScript::PeerToPeerWithMetadata
            .compiled_bytes()
            .into_vec(),
        vec![
            TransactionArgument::Address(*recipient),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U64(amount),
            TransactionArgument::U8Vector(metadata),
        ],
    )
}

/// Encode a program transferring `amount` coins from `sender` to `recipient` but pad the output
/// bytecode with unreachable instructions.
#[cfg(any(test, feature = "fuzzing"))]
pub fn encode_transfer_script_with_padding(
    recipient: &AccountAddress,
    amount: u64,
    padding_size: u64,
) -> Script {
    let mut script_mut =
        CompiledScript::deserialize(&StdlibScript::PeerToPeer.compiled_bytes().into_vec())
            .unwrap()
            .into_inner();
    script_mut
        .main
        .code
        .code
        .extend(std::iter::repeat(Bytecode::Ret).take(padding_size as usize));
    let mut script_bytes = vec![];
    script_mut
        .freeze()
        .unwrap()
        .serialize(&mut script_bytes)
        .unwrap();

    Script::new(
        script_bytes,
        vec![
            TransactionArgument::Address(*recipient),
            TransactionArgument::U8Vector(vec![]), // use empty auth key prefix
            TransactionArgument::U64(amount),
        ],
    )
}

/// Encode a program creating a fresh account at `account_address` with `initial_balance` coins
/// transferred from the sender's account balance. Fails if there is already an account at
/// `account_address` or if the sender's balance is lower than `initial_balance`.
pub fn encode_create_account_script(
    account_address: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    initial_balance: u64,
) -> Script {
    validate_auth_key_prefix(&auth_key_prefix);
    Script::new(
        StdlibScript::CreateAccount.compiled_bytes().into_vec(),
        vec![
            TransactionArgument::Address(*account_address),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U64(initial_balance),
        ],
    )
}

/// Encode a program registering the sender as a candidate validator with the given key information.
/// `network_signing_pubkey` should be a Ed25519 public key
/// `network_identity_pubkey` should be a X25519 public key
/// `consensus_pubkey` should be a Ed25519 public c=key
pub fn encode_register_validator_script(
    consensus_pubkey: Vec<u8>,
    validator_network_signing_pubkey: Vec<u8>,
    validator_network_identity_pubkey: Vec<u8>,
    validator_network_address: Vec<u8>,
    fullnodes_network_identity_pubkey: Vec<u8>,
    fullnodes_network_address: Vec<u8>,
) -> Script {
    Script::new(
        StdlibScript::RegisterValidator.compiled_bytes().into_vec(),
        vec![
            TransactionArgument::U8Vector(consensus_pubkey),
            TransactionArgument::U8Vector(validator_network_signing_pubkey),
            TransactionArgument::U8Vector(validator_network_identity_pubkey),
            TransactionArgument::U8Vector(validator_network_address),
            TransactionArgument::U8Vector(fullnodes_network_identity_pubkey),
            TransactionArgument::U8Vector(fullnodes_network_address),
        ],
    )
}

/// Encode a program adding `to_remove` to the set of pending validator removals. Fails if
/// the `to_remove` address is already in the validator set or already in the pending removals.
pub fn encode_remove_validator_script(to_remove: &AccountAddress) -> Script {
    Script::new(
        StdlibScript::RemoveValidator.compiled_bytes().into_vec(),
        vec![TransactionArgument::Address(*to_remove)],
    )
}

/// Encode a program that rotates the sender's consensus public key to `new_key`.
pub fn encode_rotate_consensus_pubkey_script(new_key: Vec<u8>) -> Script {
    Script::new(
        StdlibScript::RotateConsensusPubkey
            .compiled_bytes()
            .into_vec(),
        vec![TransactionArgument::U8Vector(new_key)],
    )
}

/// Encode a program that rotates the sender's authentication key to `new_key`. `new_key` should be
/// a 256 bit sha3 hash of an ed25519 public key.
pub fn rotate_authentication_key_script(new_hashed_key: Vec<u8>) -> Script {
    Script::new(
        StdlibScript::RotateAuthenticationKey
            .compiled_bytes()
            .into_vec(),
        vec![TransactionArgument::U8Vector(new_hashed_key)],
    )
}

// TODO: this should go away once we are no longer using it in tests
/// Encode a program creating `amount` coins for sender
pub fn encode_mint_script(
    sender: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    amount: u64,
) -> Script {
    validate_auth_key_prefix(&auth_key_prefix);
    Script::new(
        StdlibScript::Mint.compiled_bytes().into_vec(),
        vec![
            TransactionArgument::Address(*sender),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U64(amount),
        ],
    )
}

// TODO: this should go away once we are no longer using it in tests
pub fn encode_block_prologue_script(block_metadata: BlockMetadata) -> Transaction {
    Transaction::BlockMetadata(block_metadata)
}

// TODO: delete and use StdlibScript::try_from directly if it's ok to drop the "_transaction"?
/// Returns a user friendly mnemonic for the transaction type if the transaction is
/// for a known, white listed, transaction.
pub fn get_transaction_name(code: &[u8]) -> String {
    StdlibScript::try_from(code).map_or("<unknown transaction>".to_string(), |name| {
        format!("{}_transaction", name)
    })
}
