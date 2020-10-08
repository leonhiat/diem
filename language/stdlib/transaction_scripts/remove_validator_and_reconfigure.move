script {
use 0x1::LibraSystem;
use 0x1::SlidingNonce;
use 0x1::ValidatorConfig;

/// # Summary
/// This script removes a validator account from the validator set, and triggers a reconfiguration
/// of the system to remove the validator from the system. This transaction can only be
/// successfully called by the Libra Root account.
///
/// # Technical Description
/// This script removes the account at `validator_address` from the validator set. This transaction
/// emits a `LibraConfig::NewEpochEvent` event. Once the reconfiguration triggered by this event
/// has been performed, the account at `validator_address` is no longer considered to be a
/// validator in the network. This transaction will fail if the validator at `validator_address`
/// is not in the validator set.
///
/// # Parameters
/// | Name                | Type         | Description                                                                                                                        |
/// | ------              | ------       | -------------                                                                                                                      |
/// | `lr_account`        | `&signer`    | The signer reference of the sending account of this transaction. Must be the Libra Root signer.                                    |
/// | `sliding_nonce`     | `u64`        | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction.                                                         |
/// | `validator_name`    | `vector<u8>` | ASCII-encoded human name for the validator. Must match the human name in the `ValidatorConfig::ValidatorConfig` for the validator. |
/// | `validator_address` | `address`    | The validator account address to be removed from the validator set.                                                                |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                            | Description                                                                                     |
/// | ----------------           | --------------                          | -------------                                                                                   |
/// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`          | A `SlidingNonce` resource is not published under `lr_account`.                                  |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`          | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not.      |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`          | The `sliding_nonce` is too far in the future.                                                   |
/// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED` | The `sliding_nonce` has been previously recorded.                                               |
/// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`          | The sending account is not the Libra Root account or Treasury Compliance account                |
/// | 0                          | 0                                       | The provided `validator_name` does not match the already-recorded human name for the validator. |
/// | `Errors::INVALID_ARGUMENT` | `LibraSystem::ENOT_AN_ACTIVE_VALIDATOR` | The validator to be removed is not in the validator set.                                        |
/// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::ELIBRA_ROOT`            | The sending account is not the Libra Root account.                                              |
/// | `Errors::REQUIRES_ROLE`    | `Roles::ELIBRA_ROOT`                    | The sending account is not the Libra Root account.                                              |
///
/// # Related Scripts
/// * `Script::create_validator_account`
/// * `Script::create_validator_operator_account`
/// * `Script::register_validator_config`
/// * `Script::add_validator_and_reconfigure`
/// * `Script::set_validator_operator`
/// * `Script::set_validator_operator_with_nonce_admin`
/// * `Script::set_validator_config_and_reconfigure`

fun remove_validator_and_reconfigure(
    lr_account: &signer,
    sliding_nonce: u64,
    validator_name: vector<u8>,
    validator_address: address
) {
    SlidingNonce::record_nonce_or_abort(lr_account, sliding_nonce);
    // TODO: Use an error code from Errors.move
    assert(ValidatorConfig::get_human_name(validator_address) == validator_name, 0);
    LibraSystem::remove_validator(lr_account, validator_address);
}

spec fun remove_validator_and_reconfigure {
    use 0x1::LibraAccount;
    use 0x1::Errors;
    use 0x1::Roles;

    include LibraAccount::TransactionChecks{sender: lr_account}; // properties checked by the prologue.
    include SlidingNonce::RecordNonceAbortsIf{seq_nonce: sliding_nonce, account: lr_account};
    // next is due to abort in get_human_name
    include ValidatorConfig::AbortsIfNoValidatorConfig{addr: validator_address};
    // TODO: use an error code from Errors.move instead of 0.
    aborts_if ValidatorConfig::get_human_name(validator_address) != validator_name with 0;
    include LibraSystem::RemoveValidatorAbortsIf{validator_addr: validator_address};
    include LibraSystem::RemoveValidatorEnsures{validator_addr: validator_address};

    /// Reports INVALID_STATE because of is_operating() and !exists<LibraSystem::CapabilityHolder>.
    /// is_operating() is always true during transactions, and CapabilityHolder is published
    /// during initialization (Genesis).
    /// Reports REQUIRES_ROLE if lr_account is not Libra root, but that can't happen
    /// in practice because it aborts with NOT_PUBLISHED or REQUIRES_ADDRESS, first.
    aborts_with [check]
        0, // Odd error code in assert on second statement in add_validator_and_reconfigure
        Errors::INVALID_ARGUMENT,
        Errors::NOT_PUBLISHED,
        Errors::REQUIRES_ADDRESS,
        Errors::INVALID_STATE, // TODO: Undocumented error code. Can be raised in `LibraConfig::reconfigure_`.
        Errors::REQUIRES_ROLE;

    /// Access Control
    /// Only the Libra Root account can remove Validators [[H13]][PERMISSION].
    include Roles::AbortsIfNotLibraRoot{account: lr_account};
}
}
