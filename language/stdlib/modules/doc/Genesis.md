
<a name="0x1_Genesis"></a>

# Module `0x1::Genesis`

The <code><a href="Genesis.md#0x1_Genesis">Genesis</a></code> module defines the Move initialization entry point of the Libra framework
when executing from a fresh state.

> TODO: Currently there are a few additional functions called from Rust during genesis.
> Document which these are and in which order they are called.


-  [Function `initialize`](#0x1_Genesis_initialize)


<pre><code><b>use</b> <a href="AccountFreezing.md#0x1_AccountFreezing">0x1::AccountFreezing</a>;
<b>use</b> <a href="ChainId.md#0x1_ChainId">0x1::ChainId</a>;
<b>use</b> <a href="Coin1.md#0x1_Coin1">0x1::Coin1</a>;
<b>use</b> <a href="DualAttestation.md#0x1_DualAttestation">0x1::DualAttestation</a>;
<b>use</b> <a href="LBR.md#0x1_LBR">0x1::LBR</a>;
<b>use</b> <a href="Libra.md#0x1_Libra">0x1::Libra</a>;
<b>use</b> <a href="LibraAccount.md#0x1_LibraAccount">0x1::LibraAccount</a>;
<b>use</b> <a href="LibraBlock.md#0x1_LibraBlock">0x1::LibraBlock</a>;
<b>use</b> <a href="LibraConfig.md#0x1_LibraConfig">0x1::LibraConfig</a>;
<b>use</b> <a href="LibraSystem.md#0x1_LibraSystem">0x1::LibraSystem</a>;
<b>use</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp">0x1::LibraTimestamp</a>;
<b>use</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">0x1::LibraTransactionPublishingOption</a>;
<b>use</b> <a href="LibraVMConfig.md#0x1_LibraVMConfig">0x1::LibraVMConfig</a>;
<b>use</b> <a href="LibraVersion.md#0x1_LibraVersion">0x1::LibraVersion</a>;
<b>use</b> <a href="TransactionFee.md#0x1_TransactionFee">0x1::TransactionFee</a>;
</code></pre>



<a name="0x1_Genesis_initialize"></a>

## Function `initialize`

Initializes the Libra framework.


<pre><code><b>fun</b> <a href="Genesis.md#0x1_Genesis_initialize">initialize</a>(lr_account: &signer, tc_account: &signer, lr_auth_key: vector&lt;u8&gt;, tc_auth_key: vector&lt;u8&gt;, initial_script_allow_list: vector&lt;vector&lt;u8&gt;&gt;, is_open_module: bool, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, chain_id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Genesis.md#0x1_Genesis_initialize">initialize</a>(
    lr_account: &signer,
    tc_account: &signer,
    lr_auth_key: vector&lt;u8&gt;,
    tc_auth_key: vector&lt;u8&gt;,
    initial_script_allow_list: vector&lt;vector&lt;u8&gt;&gt;,
    is_open_module: bool,
    instruction_schedule: vector&lt;u8&gt;,
    native_schedule: vector&lt;u8&gt;,
    chain_id: u8,
) {

    <a href="LibraAccount.md#0x1_LibraAccount_initialize">LibraAccount::initialize</a>(lr_account, x"00000000000000000000000000000000");

    <a href="ChainId.md#0x1_ChainId_initialize">ChainId::initialize</a>(lr_account, chain_id);

    // On-chain config setup
    <a href="LibraConfig.md#0x1_LibraConfig_initialize">LibraConfig::initialize</a>(lr_account);

    // Currency setup
    <a href="Libra.md#0x1_Libra_initialize">Libra::initialize</a>(lr_account);

    // Currency setup
    <a href="Coin1.md#0x1_Coin1_initialize">Coin1::initialize</a>(lr_account, tc_account);

    <a href="LBR.md#0x1_LBR_initialize">LBR::initialize</a>(
        lr_account,
        tc_account,
    );

    <a href="AccountFreezing.md#0x1_AccountFreezing_initialize">AccountFreezing::initialize</a>(lr_account);

    <a href="TransactionFee.md#0x1_TransactionFee_initialize">TransactionFee::initialize</a>(tc_account);

    <a href="LibraSystem.md#0x1_LibraSystem_initialize_validator_set">LibraSystem::initialize_validator_set</a>(
        lr_account,
    );
    <a href="LibraVersion.md#0x1_LibraVersion_initialize">LibraVersion::initialize</a>(
        lr_account,
    );
    <a href="DualAttestation.md#0x1_DualAttestation_initialize">DualAttestation::initialize</a>(
        lr_account,
    );
    <a href="LibraBlock.md#0x1_LibraBlock_initialize_block_metadata">LibraBlock::initialize_block_metadata</a>(lr_account);

    <b>let</b> lr_rotate_key_cap = <a href="LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(lr_account);
    <a href="LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&lr_rotate_key_cap, lr_auth_key);
    <a href="LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(lr_rotate_key_cap);

    <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_initialize">LibraTransactionPublishingOption::initialize</a>(
        lr_account,
        initial_script_allow_list,
        is_open_module,
    );

    <a href="LibraVMConfig.md#0x1_LibraVMConfig_initialize">LibraVMConfig::initialize</a>(
        lr_account,
        instruction_schedule,
        native_schedule,
    );

    <b>let</b> tc_rotate_key_cap = <a href="LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(tc_account);
    <a href="LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(&tc_rotate_key_cap, tc_auth_key);
    <a href="LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">LibraAccount::restore_key_rotation_capability</a>(tc_rotate_key_cap);

    // After we have called this function, all invariants which are guarded by
    // `<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; ...` will become active and a verification condition.
    // See also discussion at function specification.
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_set_time_has_started">LibraTimestamp::set_time_has_started</a>(lr_account);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>

For verification of genesis, the goal is to prove that all the invariants which
become active after the end of this function hold. This cannot be achieved with
modular verification as we do in regular continuous testing. Rather, this module must
be verified **together** with the module(s) which provides the invariant.

> TODO: currently verifying this module together with modules providing invariants
> (see above) times out. This can likely be solved by making more of the initialize
> functions called by this function opaque, and prove the according invariants locally to
> each module.

Assume that this is called in genesis state (no timestamp).


<pre><code><b>requires</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>();
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/libra/lip/blob/master/lips/lip-2.md
[ROLE]: https://github.com/libra/lip/blob/master/lips/lip-2.md#roles
[PERMISSION]: https://github.com/libra/lip/blob/master/lips/lip-2.md#permissions
