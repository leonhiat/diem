// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use language_e2e_tests::{test_with_different_versions, versioning::CURRENT_RELEASE_VERSIONS};

use diem_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use diem_types::{
    account_config,
    transaction::{authenticator::AuthenticationKey, Script, TransactionArgument},
    vm_status::StatusCode,
};

use compiler::Compiler;
use diem_types::transaction::WriteSetPayload;

#[test]
fn admin_script_rotate_key_single_signer_no_epoch() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let new_account = executor.create_raw_account_data(100_000, 0);
        executor.add_account_data(&new_account);

        let privkey = Ed25519PrivateKey::generate_for_testing();
        let pubkey = privkey.public_key();
        let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();

        let script_body = {
            let code = r#"
import 0x1.DiemAccount;

main(dr_account: signer, account: signer, auth_key_prefix: vector<u8>) {
  let rotate_cap: DiemAccount.KeyRotationCapability;
  rotate_cap = DiemAccount.extract_key_rotation_capability(&account);
  DiemAccount.rotate_authentication_key(&rotate_cap, move(auth_key_prefix));
  DiemAccount.restore_key_rotation_capability(move(rotate_cap));

  return;
}
"#;

            let compiler = Compiler {
                address: account_config::CORE_CODE_ADDRESS,
                skip_stdlib_deps: false,
                extra_deps: vec![],
            };
            compiler
                .into_script_blob("file_name", code)
                .expect("Failed to compile")
        };
        let account = test_env.dr_account;
        let txn = account
            .transaction()
            .write_set(WriteSetPayload::Script {
                script: Script::new(
                    script_body,
                    vec![],
                    vec![TransactionArgument::U8Vector(new_key_hash.clone())],
                ),
                execute_as: *new_account.address(),
            })
            .sequence_number(test_env.dr_sequence_number)
            .sign();
        executor.new_block();
        let output = executor.execute_and_apply(txn);

        // The transaction should not trigger a reconfiguration.
        let new_epoch_event_key = diem_types::on_chain_config::new_epoch_event_key();
        assert!(!output
            .events()
            .iter()
            .any(|event| *event.key() == new_epoch_event_key));

        let updated_sender = executor
            .read_account_resource(new_account.account())
            .expect("sender must exist");

        assert_eq!(updated_sender.authentication_key(), new_key_hash.as_slice());
    }
    }
}

#[test]
fn admin_script_rotate_key_single_signer_new_epoch() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let new_account = executor.create_raw_account_data(100_000, 0);
        executor.add_account_data(&new_account);

        let privkey = Ed25519PrivateKey::generate_for_testing();
        let pubkey = privkey.public_key();
        let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();

        let script_body = {
            let code = r#"
import 0x1.DiemAccount;
import 0x1.DiemConfig;

main(dr_account: signer, account: signer, auth_key_prefix: vector<u8>) {
  let rotate_cap: DiemAccount.KeyRotationCapability;
  rotate_cap = DiemAccount.extract_key_rotation_capability(&account);
  DiemAccount.rotate_authentication_key(&rotate_cap, move(auth_key_prefix));
  DiemAccount.restore_key_rotation_capability(move(rotate_cap));

  DiemConfig.reconfigure(&dr_account);
  return;
}
"#;

            let compiler = Compiler {
                address: account_config::CORE_CODE_ADDRESS,
                skip_stdlib_deps: false,
                extra_deps: vec![],
            };
            compiler
                .into_script_blob("file_name", code)
                .expect("Failed to compile")
        };
        let account = test_env.dr_account;
        let txn = account
            .transaction()
            .write_set(WriteSetPayload::Script {
                script: Script::new(
                    script_body,
                    vec![],
                    vec![TransactionArgument::U8Vector(new_key_hash.clone())],
                ),
                execute_as: *new_account.address(),
            })
            .sequence_number(test_env.dr_sequence_number)
            .sign();
        executor.new_block();
        let output = executor.execute_and_apply(txn);

        // The transaction should trigger a reconfiguration.
        let new_epoch_event_key = diem_types::on_chain_config::new_epoch_event_key();
        assert!(output
            .events()
            .iter()
            .any(|event| *event.key() == new_epoch_event_key));

        let updated_sender = executor
            .read_account_resource(new_account.account())
            .expect("sender must exist");

        assert_eq!(updated_sender.authentication_key(), new_key_hash.as_slice());
    }
    }
}

#[test]
fn admin_script_rotate_key_multi_signer() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let new_account = executor.create_raw_account_data(100_000, 0);
        executor.add_account_data(&new_account);

        let privkey = Ed25519PrivateKey::generate_for_testing();
        let pubkey = privkey.public_key();
        let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();

        let script_body = {
            let code = r#"
import 0x1.DiemAccount;

main(account: signer, auth_key_prefix: vector<u8>) {
  let rotate_cap: DiemAccount.KeyRotationCapability;
  rotate_cap = DiemAccount.extract_key_rotation_capability(&account);
  DiemAccount.rotate_authentication_key(&rotate_cap, move(auth_key_prefix));
  DiemAccount.restore_key_rotation_capability(move(rotate_cap));

  return;
}
"#;

            let compiler = Compiler {
                address: account_config::CORE_CODE_ADDRESS,
                skip_stdlib_deps: false,
                extra_deps: vec![],
            };
            compiler
                .into_script_blob("file_name", code)
                .expect("Failed to compile")
        };
        let account = test_env.dr_account;
        let txn = account
            .transaction()
            .write_set(WriteSetPayload::Script {
                script: Script::new(
                    script_body,
                    vec![],
                    vec![TransactionArgument::U8Vector(new_key_hash)],
                ),
                execute_as: *new_account.address(),
            })
            .sequence_number(test_env.dr_sequence_number)
            .sign();
        executor.new_block();
        let output = executor.execute_transaction(txn);
        assert_eq!(output.status().status(), Err(StatusCode::INVALID_WRITE_SET));
    }
    }
}
