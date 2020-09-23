script {
use 0x1::LibraAccount;

/// # Summary
/// Adds a zero `Currency` balance to the sending `account`. This will enable `account` to
/// send, receive, and hold `Libra::Libra<Currency>` coins. This transaction can be
/// successfully sent by any account that is allowed to hold balances
/// (e.g., VASP, Designated Dealer).
///
/// # Technical Description
/// After the successful execution of this transaction the sending account will have a
/// `LibraAccount::Balance<Currency>` resource with zero balance published under it. Only
/// accounts that can hold balances can send this transaction, the sending account cannot
/// already have a `LibraAccount::Balance<Currency>` published under it.
///
/// # Parameters
/// | Name       | Type      | Description                                                                                                                                         |
/// | ------     | ------    | -------------                                                                                                                                       |
/// | `Currency` | Type      | The Move type for the `Currency` being added to the sending account of the transaction. `Currency` must be an already-registered currency on-chain. |
/// | `account`  | `&signer` | The signer of the sending account of the transaction.                                                                                               |
///
/// # Common Abort Conditions
/// | Error Category              | Error Reason                             | Description                                                                |
/// | ----------------            | --------------                           | -------------                                                              |
/// | `Errors::NOT_PUBLISHED`     | `Libra::ECURRENCY_INFO`                  | The `Currency` is not a registered currency on-chain.                      |
/// | `Errors::INVALID_ARGUMENT`  | `LibraAccount::EROLE_CANT_STORE_BALANCE` | The sending `account`'s role does not permit balances.                     |
/// | `Errors::ALREADY_PUBLISHED` | `LibraAccount::EADD_EXISTING_CURRENCY`   | A balance for `Currency` is already published under the sending `account`. |
///
/// # Related Scripts
/// * `Script::create_child_vasp_account`
/// * `Script::create_parent_vasp_account`
/// * `Script::peer_to_peer_with_metadata`

fun add_currency_to_account<Currency>(account: &signer) {
    LibraAccount::add_currency<Currency>(account);
}
spec fun add_currency_to_account {
    use 0x1::Errors;

    include LibraAccount::AddCurrencyAbortsIf<Currency>;
    include LibraAccount::AddCurrencyEnsures<Currency>;

    aborts_with [check]
        Errors::NOT_PUBLISHED,
        Errors::INVALID_ARGUMENT,
        Errors::ALREADY_PUBLISHED;
}
}
