//! account: alice, 0, 0, address
//! account: bob, 0, 0, address
//! account: richie, 10Coin1
//! account: sally, 0, 0, address

// BEGIN: registration of a currency
//! account: validator, 1000000, 0, validator

//! new-transaction
//! sender: libraroot
// Change option to CustomModule
script {
use 0x1::LibraTransactionPublishingOption;
fun main(config: &signer) {
    LibraTransactionPublishingOption::set_open_module(config, false)
}
}
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: validator
//! block-time: 3


//! new-transaction
//! sender: libraroot
address 0x1 {
module COIN {
    use 0x1::FixedPoint32;
    use 0x1::Libra;

    struct COIN { }

    public fun initialize(lr_account: &signer, tc_account: &signer) {
        // Register the COIN currency.
        Libra::register_SCS_currency<COIN>(
            lr_account,
            tc_account,
            FixedPoint32::create_from_rational(1, 2), // exchange rate to LBR
            1000000, // scaling_factor = 10^6
            100,     // fractional_part = 10^2
            b"COIN",
        )
    }
}
}
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: validator
//! block-time: 4

//! new-transaction
//! sender: libraroot
//! execute-as: blessed
script {
use 0x1::TransactionFee;
use 0x1::COIN::{Self, COIN};
fun main(lr_account: &signer, tc_account: &signer) {
    COIN::initialize(lr_account, tc_account);
    TransactionFee::add_txn_fee_currency<COIN>(tc_account);
}
}
// check: "Keep(EXECUTED)"

// END: registration of a currency

//! new-transaction
//! sender: blessed
//! type-args: 0x1::COIN::COIN
//! args: 0, {{sally}}, {{sally::auth_key}}, b"bob", false
stdlib_script::create_designated_dealer
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
script {
use 0x1::COIN::COIN;
use 0x1::LibraAccount;
fun main(account: &signer) {
    LibraAccount::tiered_mint<COIN>(account, {{sally}}, 10, 3);
}
}

// create parent VASP accounts for alice and bob
//! new-transaction
//! sender: blessed
script {
use 0x1::Coin1::Coin1;
use 0x1::COIN::COIN;
use 0x1::LibraAccount;
fun main(tc_account: &signer) {
    let add_all_currencies = false;

    LibraAccount::create_parent_vasp_account<Coin1>(
        tc_account,
        {{alice}},
        {{alice::auth_key}},
        x"A1",
        add_all_currencies,
    );

    LibraAccount::create_parent_vasp_account<COIN>(
        tc_account,
        {{bob}},
        {{bob::auth_key}},
        x"B1",
        add_all_currencies,
    );
}
}
// check: "Keep(EXECUTED)"

// Give alice money from richie
//! new-transaction
//! sender: richie
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, 10, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"

// Give bob money from sally
//! new-transaction
//! sender: sally
script {
use 0x1::LibraAccount;
use 0x1::COIN::COIN;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<COIN>(&with_cap, {{bob}}, 10, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use 0x1::LibraAccount;
use 0x1::COIN::COIN;
fun main(account: &signer) {
    LibraAccount::add_currency<COIN>(account);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin1>(account);
}
}
// check: "Keep(EXECUTED)"

// Adding a bogus currency should abort
//! new-transaction
//! sender: alice
script {
use 0x1::LibraAccount;
fun main(account: &signer) {
    LibraAccount::add_currency<u64>(account);
}
}
// check: "Keep(ABORTED { code: 261,"

// Adding Coin1 a second time should fail with ADD_EXISTING_CURRENCY
//! new-transaction
//! sender: alice
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin1>(account);
}
}
// check: "Keep(ABORTED { code: 3846,"

//! new-transaction
//! sender: alice
script {
use 0x1::LibraAccount;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin1>(&with_cap, {{bob}}, 10, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
    assert(LibraAccount::balance<Coin1>({{alice}}) == 0, 0);
    assert(LibraAccount::balance<Coin1>({{bob}}) == 10, 1);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
use 0x1::LibraAccount;
use 0x1::COIN::COIN;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<COIN>(&with_cap, {{alice}}, 10, x"", x"");
    LibraAccount::pay_from<Coin1>(&with_cap, {{alice}}, 10, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);
    assert(LibraAccount::balance<Coin1>({{bob}}) == 0, 2);
    assert(LibraAccount::balance<COIN>({{bob}}) == 0, 3);
    assert(LibraAccount::balance<Coin1>({{alice}}) == 10, 4);
    assert(LibraAccount::balance<COIN>({{alice}}) == 10, 5);
}
}
// check: "Keep(EXECUTED)"
