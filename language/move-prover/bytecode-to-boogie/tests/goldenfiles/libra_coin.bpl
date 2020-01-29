

// ** structs of module LibraCoin

const unique LibraCoin_T: TypeName;
const LibraCoin_T_value: FieldName;
axiom LibraCoin_T_value == 0;
function LibraCoin_T_type_value(): TypeValue {
    StructType(LibraCoin_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
procedure {:inline 1} Pack_LibraCoin_T(value: Value) returns (_struct: Value)
{
    assume IsValidU64(value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, value));
}

procedure {:inline 1} Unpack_LibraCoin_T(_struct: Value) returns (value: Value)
{
    assume is#Vector(_struct);
    value := SelectField(_struct, LibraCoin_T_value);
    assume IsValidU64(value);
}

const unique LibraCoin_MintCapability: TypeName;
const LibraCoin_MintCapability__dummy: FieldName;
axiom LibraCoin_MintCapability__dummy == 0;
function LibraCoin_MintCapability_type_value(): TypeValue {
    StructType(LibraCoin_MintCapability, ExtendTypeValueArray(EmptyTypeValueArray, BooleanType()))
}
procedure {:inline 1} Pack_LibraCoin_MintCapability(_dummy: Value) returns (_struct: Value)
{
    assume is#Boolean(_dummy);
    _struct := Vector(ExtendValueArray(EmptyValueArray, _dummy));
}

procedure {:inline 1} Unpack_LibraCoin_MintCapability(_struct: Value) returns (_dummy: Value)
{
    assume is#Vector(_struct);
    _dummy := SelectField(_struct, LibraCoin_MintCapability__dummy);
    assume is#Boolean(_dummy);
}

const unique LibraCoin_MarketCap: TypeName;
const LibraCoin_MarketCap_total_value: FieldName;
axiom LibraCoin_MarketCap_total_value == 0;
function LibraCoin_MarketCap_type_value(): TypeValue {
    StructType(LibraCoin_MarketCap, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
procedure {:inline 1} Pack_LibraCoin_MarketCap(total_value: Value) returns (_struct: Value)
{
    assume IsValidU64(total_value);
    _struct := Vector(ExtendValueArray(EmptyValueArray, total_value));
}

procedure {:inline 1} Unpack_LibraCoin_MarketCap(_struct: Value) returns (total_value: Value)
{
    assume is#Vector(_struct);
    total_value := SelectField(_struct, LibraCoin_MarketCap_total_value);
    assume IsValidU64(total_value);
}



// ** functions of module LibraCoin

procedure {:inline 1} LibraCoin_mint_with_default_capability (amount: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value), Integer(i#Integer(old(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))) + i#Integer(amount)))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), amount)));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(amount) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(amount) + i#Integer(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))))) || b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(amount) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(amount) + i#Integer(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var __t1: Value; // IntegerType()
    var __t2: Value; // AddressType()
    var __t3: Reference; // ReferenceType(LibraCoin_MintCapability_type_value())
    var __t4: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#LibraCoin#mint_with_default_capability#0#amount: [Position]Value;
    var debug#LibraCoin#mint_with_default_capability#1#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;
    debug#LibraCoin#mint_with_default_capability#0#amount := EmptyPositionMap;
    debug#LibraCoin#mint_with_default_capability#1#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 0, amount);
    debug#LibraCoin#mint_with_default_capability#0#amount := debug#LibraCoin#mint_with_default_capability#0#amount[Position(806) := amount];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := BorrowGlobal(GetLocal(__m, __frame + 2), LibraCoin_MintCapability_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call __t4 := LibraCoin_mint(GetLocal(__m, __frame + 1), __t3);
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    __ret0 := GetLocal(__m, __frame + 4);
    debug#LibraCoin#mint_with_default_capability#1#__ret := debug#LibraCoin#mint_with_default_capability#1#__ret[Position(1401) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#LibraCoin#mint_with_default_capability#1#__ret := debug#LibraCoin#mint_with_default_capability#1#__ret[Position(1486) := __ret0];
}

procedure LibraCoin_mint_with_default_capability_verify (amount: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraCoin_mint_with_default_capability(amount);
}

procedure {:inline 1} LibraCoin_mint (value: Value, capability: Reference) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value), Integer(i#Integer(old(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))) + i#Integer(value)))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), value)));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(value) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(value) + i#Integer(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))) || b#Boolean(Boolean(i#Integer(value) > i#Integer(Integer(1000000000000000)))) || b#Boolean(Boolean(i#Integer(Integer(i#Integer(value) + i#Integer(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value)))) > i#Integer(Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var market_cap_ref: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var market_cap_total_value: Value; // IntegerType()
    var __t4: Reference; // ReferenceType(LibraCoin_MintCapability_type_value())
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // BooleanType()
    var __t10: Value; // BooleanType()
    var __t11: Value; // IntegerType()
    var __t12: Value; // AddressType()
    var __t13: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var __t14: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var __t15: Reference; // ReferenceType(IntegerType())
    var __t16: Value; // IntegerType()
    var __t17: Value; // IntegerType()
    var __t18: Value; // IntegerType()
    var __t19: Value; // IntegerType()
    var __t20: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var __t21: Reference; // ReferenceType(IntegerType())
    var __t22: Value; // IntegerType()
    var __t23: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#LibraCoin#mint#0#value: [Position]Value;
    var debug#LibraCoin#mint#1#capability: [Position]Value;
    var debug#LibraCoin#mint#2#market_cap_ref: [Position]Value;
    var debug#LibraCoin#mint#3#market_cap_total_value: [Position]Value;
    var debug#LibraCoin#mint#4#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 24;
    debug#LibraCoin#mint#0#value := EmptyPositionMap;
    debug#LibraCoin#mint#1#capability := EmptyPositionMap;
    debug#LibraCoin#mint#2#market_cap_ref := EmptyPositionMap;
    debug#LibraCoin#mint#3#market_cap_total_value := EmptyPositionMap;
    debug#LibraCoin#mint#4#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(value);
    __m := UpdateLocal(__m, __frame + 0, value);
    debug#LibraCoin#mint#0#value := debug#LibraCoin#mint#0#value[Position(1723) := value];
    assume is#Vector(Dereference(__m, capability));
    assume IsValidReferenceParameter(__m, __frame, capability);
    debug#LibraCoin#mint#1#capability := debug#LibraCoin#mint#1#capability[Position(1723) := Dereference(__m, capability)];

    // bytecode translation starts here
    call __t4 := CopyOrMoveRef(capability);

    // unimplemented instruction: NoOp

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(1000000000);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := LdConst(1000000);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := Le(GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __tmp := GetLocal(__m, __frame + 10);
    if (!b#Boolean(__tmp)) { goto Label_11; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    goto Label_Abort;

Label_11:
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __t13 := BorrowGlobal(GetLocal(__m, __frame + 12), LibraCoin_MarketCap_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call market_cap_ref := CopyOrMoveRef(__t13);

    call __t14 := CopyOrMoveRef(market_cap_ref);

    call __t15 := BorrowField(__t14, LibraCoin_MarketCap_total_value);

    call __tmp := ReadRef(__t15);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    debug#LibraCoin#mint#3#market_cap_total_value := debug#LibraCoin#mint#3#market_cap_total_value[Position(2964) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 17), GetLocal(__m, __frame + 18));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __t20 := CopyOrMoveRef(market_cap_ref);

    call __t21 := BorrowField(__t20, LibraCoin_MarketCap_total_value);

    call WriteRef(__t21, GetLocal(__m, __frame + 19));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    call __tmp := Pack_LibraCoin_T(GetLocal(__m, __frame + 22));
    __m := UpdateLocal(__m, __frame + 23, __tmp);

    __ret0 := GetLocal(__m, __frame + 23);
    debug#LibraCoin#mint#4#__ret := debug#LibraCoin#mint#4#__ret[Position(3129) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#LibraCoin#mint#4#__ret := debug#LibraCoin#mint#4#__ret[Position(3163) := __ret0];
}

procedure LibraCoin_mint_verify (value: Value, capability: Reference) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraCoin_mint(value, capability);
}

procedure {:inline 1} LibraCoin_initialize () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn)))));
ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(__txn)))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(__txn))))), LibraCoin_MarketCap_total_value), Integer(0))));
ensures old(!(b#Boolean(Boolean(!IsEqual(Address(TxnSenderAddress(__txn)), Address(173345816)))) || b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn))))) || b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(__txn))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!IsEqual(Address(TxnSenderAddress(__txn)), Address(173345816)))) || b#Boolean(ExistsResource(__m, LibraCoin_MintCapability_type_value(), a#Address(Address(TxnSenderAddress(__txn))))) || b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(TxnSenderAddress(__txn)))))) ==> __abort_flag;

{
    // declare local variables
    var __t0: Value; // AddressType()
    var __t1: Value; // AddressType()
    var __t2: Value; // BooleanType()
    var __t3: Value; // BooleanType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // LibraCoin_MintCapability_type_value()
    var __t7: Value; // IntegerType()
    var __t8: Value; // LibraCoin_MarketCap_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 9;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := GetTxnSenderAddress();
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 0), GetLocal(__m, __frame + 1)));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __tmp := GetLocal(__m, __frame + 3);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    goto Label_Abort;

Label_7:
    call __tmp := LdTrue();
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Pack_LibraCoin_MintCapability(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call MoveToSender(LibraCoin_MintCapability_type_value(), GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := Pack_LibraCoin_MarketCap(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call MoveToSender(LibraCoin_MarketCap_type_value(), GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraCoin_initialize_verify () returns ()
{
    call InitVerification();
    call LibraCoin_initialize();
}

procedure {:inline 1} LibraCoin_market_cap () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(__ret0, SelectField(Dereference(__m, GetResourceReference(LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))), LibraCoin_MarketCap_total_value))));
ensures old(!(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816))))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!(b#Boolean(ExistsResource(__m, LibraCoin_MarketCap_type_value(), a#Address(Address(173345816)))))))) ==> __abort_flag;

{
    // declare local variables
    var __t0: Value; // AddressType()
    var __t1: Reference; // ReferenceType(LibraCoin_MarketCap_type_value())
    var __t2: Reference; // ReferenceType(IntegerType())
    var __t3: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#LibraCoin#market_cap#0#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;
    debug#LibraCoin#market_cap#0#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdAddr(173345816);
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t1 := BorrowGlobal(GetLocal(__m, __frame + 0), LibraCoin_MarketCap_type_value());
    if (__abort_flag) { goto Label_Abort; }

    call __t2 := BorrowField(__t1, LibraCoin_MarketCap_total_value);

    call __tmp := ReadRef(__t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    debug#LibraCoin#market_cap#0#__ret := debug#LibraCoin#market_cap#0#__ret[Position(4289) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#LibraCoin#market_cap#0#__ret := debug#LibraCoin#market_cap#0#__ret[Position(4353) := __ret0];
}

procedure LibraCoin_market_cap_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraCoin_market_cap();
}

procedure {:inline 1} LibraCoin_zero () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), Integer(0))));
{
    // declare local variables
    var __t0: Value; // IntegerType()
    var __t1: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#LibraCoin#zero#0#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 2;
    debug#LibraCoin#zero#0#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := Pack_LibraCoin_T(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    debug#LibraCoin#zero#0#__ret := debug#LibraCoin#zero#0#__ret[Position(4477) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#LibraCoin#zero#0#__ret := debug#LibraCoin#zero#0#__ret[Position(4501) := __ret0];
}

procedure LibraCoin_zero_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraCoin_zero();
}

procedure {:inline 1} LibraCoin_value (coin_ref: Reference) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, SelectField(Dereference(__m, coin_ref), LibraCoin_T_value))));
{
    // declare local variables
    var __t1: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t2: Reference; // ReferenceType(IntegerType())
    var __t3: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#LibraCoin#value#0#coin_ref: [Position]Value;
    var debug#LibraCoin#value#1#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;
    debug#LibraCoin#value#0#coin_ref := EmptyPositionMap;
    debug#LibraCoin#value#1#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(Dereference(__m, coin_ref));
    assume IsValidReferenceParameter(__m, __frame, coin_ref);
    debug#LibraCoin#value#0#coin_ref := debug#LibraCoin#value#0#coin_ref[Position(4555) := Dereference(__m, coin_ref)];

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(coin_ref);

    call __t2 := BorrowField(__t1, LibraCoin_T_value);

    call __tmp := ReadRef(__t2);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    debug#LibraCoin#value#1#__ret := debug#LibraCoin#value#1#__ret[Position(4644) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#LibraCoin#value#1#__ret := debug#LibraCoin#value#1#__ret[Position(4679) := __ret0];
}

procedure LibraCoin_value_verify (coin_ref: Reference) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraCoin_value(coin_ref);
}

procedure {:inline 1} LibraCoin_split (coin: Value, amount: Value) returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(b#Boolean(Boolean(IsEqual(SelectField(__ret1, LibraCoin_T_value), amount))) && b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), Integer(i#Integer(old(SelectField(coin, LibraCoin_T_value))) - i#Integer(amount)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(SelectField(coin, LibraCoin_T_value)) < i#Integer(amount))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(SelectField(coin, LibraCoin_T_value)) < i#Integer(amount)))) ==> __abort_flag;

{
    // declare local variables
    var other: Value; // LibraCoin_T_type_value()
    var __t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t4: Value; // IntegerType()
    var __t5: Value; // LibraCoin_T_type_value()
    var __t6: Value; // LibraCoin_T_type_value()
    var __t7: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#LibraCoin#split#0#coin: [Position]Value;
    var debug#LibraCoin#split#1#amount: [Position]Value;
    var debug#LibraCoin#split#2#other: [Position]Value;
    var debug#LibraCoin#split#3#__ret: [Position]Value;
    var debug#LibraCoin#split#4#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 8;
    debug#LibraCoin#split#0#coin := EmptyPositionMap;
    debug#LibraCoin#split#1#amount := EmptyPositionMap;
    debug#LibraCoin#split#2#other := EmptyPositionMap;
    debug#LibraCoin#split#3#__ret := EmptyPositionMap;
    debug#LibraCoin#split#4#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(coin);
    __m := UpdateLocal(__m, __frame + 0, coin);
    debug#LibraCoin#split#0#coin := debug#LibraCoin#split#0#coin[Position(4818) := coin];
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    debug#LibraCoin#split#1#amount := debug#LibraCoin#split#1#amount[Position(4818) := amount];

    // bytecode translation starts here
    call __t3 := BorrowLoc(__frame + 0);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __t5 := LibraCoin_withdraw(__t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t5);

    __m := UpdateLocal(__m, __frame + 5, __t5);
    debug#LibraCoin#split#0#coin := debug#LibraCoin#split#0#coin[Position(5045) := GetLocal(__m, __frame + 0)];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    debug#LibraCoin#split#2#other := debug#LibraCoin#split#2#other[Position(5037) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __ret0 := GetLocal(__m, __frame + 6);
    debug#LibraCoin#split#3#__ret := debug#LibraCoin#split#3#__ret[Position(5093) := __ret0];
    __ret1 := GetLocal(__m, __frame + 7);
    debug#LibraCoin#split#4#__ret := debug#LibraCoin#split#4#__ret[Position(5093) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#LibraCoin#split#3#__ret := debug#LibraCoin#split#3#__ret[Position(5129) := __ret0];
    __ret1 := DefaultValue;
    debug#LibraCoin#split#4#__ret := debug#LibraCoin#split#4#__ret[Position(5129) := __ret1];
}

procedure LibraCoin_split_verify (coin: Value, amount: Value) returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := LibraCoin_split(coin, amount);
}

procedure {:inline 1} LibraCoin_withdraw (coin_ref: Reference, amount: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value), Integer(i#Integer(old(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value))) - i#Integer(amount)))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), amount)));
ensures old(!(b#Boolean(Boolean(i#Integer(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value)) < i#Integer(amount))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value)) < i#Integer(amount)))) ==> __abort_flag;

{
    // declare local variables
    var value: Value; // IntegerType()
    var __t3: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t4: Reference; // ReferenceType(IntegerType())
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // BooleanType()
    var __t9: Value; // BooleanType()
    var __t10: Value; // IntegerType()
    var __t11: Value; // IntegerType()
    var __t12: Value; // IntegerType()
    var __t13: Value; // IntegerType()
    var __t14: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t15: Reference; // ReferenceType(IntegerType())
    var __t16: Value; // IntegerType()
    var __t17: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#LibraCoin#withdraw#0#coin_ref: [Position]Value;
    var debug#LibraCoin#withdraw#1#amount: [Position]Value;
    var debug#LibraCoin#withdraw#2#value: [Position]Value;
    var debug#LibraCoin#withdraw#3#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 18;
    debug#LibraCoin#withdraw#0#coin_ref := EmptyPositionMap;
    debug#LibraCoin#withdraw#1#amount := EmptyPositionMap;
    debug#LibraCoin#withdraw#2#value := EmptyPositionMap;
    debug#LibraCoin#withdraw#3#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(Dereference(__m, coin_ref));
    assume IsValidReferenceParameter(__m, __frame, coin_ref);
    debug#LibraCoin#withdraw#0#coin_ref := debug#LibraCoin#withdraw#0#coin_ref[Position(5391) := Dereference(__m, coin_ref)];
    assume IsValidU64(amount);
    __m := UpdateLocal(__m, __frame + 1, amount);
    debug#LibraCoin#withdraw#1#amount := debug#LibraCoin#withdraw#1#amount[Position(5391) := amount];

    // bytecode translation starts here
    call __t3 := CopyOrMoveRef(coin_ref);

    call __t4 := BorrowField(__t3, LibraCoin_T_value);

    call __tmp := ReadRef(__t4);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    debug#LibraCoin#withdraw#2#value := debug#LibraCoin#withdraw#2#value[Position(5692) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := Ge(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __tmp := GetLocal(__m, __frame + 9);
    if (!b#Boolean(__tmp)) { goto Label_11; }

    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    goto Label_Abort;

Label_11:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 12));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __t14 := CopyOrMoveRef(coin_ref);

    call __t15 := BorrowField(__t14, LibraCoin_T_value);

    call WriteRef(__t15, GetLocal(__m, __frame + 13));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := Pack_LibraCoin_T(GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    __ret0 := GetLocal(__m, __frame + 17);
    debug#LibraCoin#withdraw#3#__ret := debug#LibraCoin#withdraw#3#__ret[Position(5881) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#LibraCoin#withdraw#3#__ret := debug#LibraCoin#withdraw#3#__ret[Position(5916) := __ret0];
}

procedure LibraCoin_withdraw_verify (coin_ref: Reference, amount: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraCoin_withdraw(coin_ref, amount);
}

procedure {:inline 1} LibraCoin_join (coin1: Value, coin2: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(__ret0, LibraCoin_T_value), Integer(i#Integer(old(SelectField(coin1, LibraCoin_T_value))) + i#Integer(old(SelectField(coin2, LibraCoin_T_value)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(coin1, LibraCoin_T_value)) + i#Integer(SelectField(coin2, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(coin1, LibraCoin_T_value)) + i#Integer(SelectField(coin2, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t3: Value; // LibraCoin_T_type_value()
    var __t4: Value; // LibraCoin_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#LibraCoin#join#0#coin1: [Position]Value;
    var debug#LibraCoin#join#1#coin2: [Position]Value;
    var debug#LibraCoin#join#2#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;
    debug#LibraCoin#join#0#coin1 := EmptyPositionMap;
    debug#LibraCoin#join#1#coin2 := EmptyPositionMap;
    debug#LibraCoin#join#2#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(coin1);
    __m := UpdateLocal(__m, __frame + 0, coin1);
    debug#LibraCoin#join#0#coin1 := debug#LibraCoin#join#0#coin1[Position(6020) := coin1];
    assume is#Vector(coin2);
    __m := UpdateLocal(__m, __frame + 1, coin2);
    debug#LibraCoin#join#1#coin2 := debug#LibraCoin#join#1#coin2[Position(6020) := coin2];

    // bytecode translation starts here
    call __t2 := BorrowLoc(__frame + 0);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call LibraCoin_deposit(__t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }
    debug#LibraCoin#join#0#coin1 := debug#LibraCoin#join#0#coin1[Position(6215) := GetLocal(__m, __frame + 0)];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    debug#LibraCoin#join#2#__ret := debug#LibraCoin#join#2#__ret[Position(6262) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#LibraCoin#join#2#__ret := debug#LibraCoin#join#2#__ret[Position(6286) := __ret0];
}

procedure LibraCoin_join_verify (coin1: Value, coin2: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := LibraCoin_join(coin1, coin2);
}

procedure {:inline 1} LibraCoin_deposit (coin_ref: Reference, check: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value), Integer(i#Integer(old(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value))) + i#Integer(old(SelectField(check, LibraCoin_T_value)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value)) + i#Integer(SelectField(check, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(SelectField(Dereference(__m, coin_ref), LibraCoin_T_value)) + i#Integer(SelectField(check, LibraCoin_T_value)))) > i#Integer(Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var value: Value; // IntegerType()
    var check_value: Value; // IntegerType()
    var __t4: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t5: Reference; // ReferenceType(IntegerType())
    var __t6: Value; // IntegerType()
    var __t7: Value; // LibraCoin_T_type_value()
    var __t8: Value; // IntegerType()
    var __t9: Value; // IntegerType()
    var __t10: Value; // IntegerType()
    var __t11: Value; // IntegerType()
    var __t12: Reference; // ReferenceType(LibraCoin_T_type_value())
    var __t13: Reference; // ReferenceType(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#LibraCoin#deposit#0#coin_ref: [Position]Value;
    var debug#LibraCoin#deposit#1#check: [Position]Value;
    var debug#LibraCoin#deposit#2#value: [Position]Value;
    var debug#LibraCoin#deposit#3#check_value: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 14;
    debug#LibraCoin#deposit#0#coin_ref := EmptyPositionMap;
    debug#LibraCoin#deposit#1#check := EmptyPositionMap;
    debug#LibraCoin#deposit#2#value := EmptyPositionMap;
    debug#LibraCoin#deposit#3#check_value := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(Dereference(__m, coin_ref));
    assume IsValidReferenceParameter(__m, __frame, coin_ref);
    debug#LibraCoin#deposit#0#coin_ref := debug#LibraCoin#deposit#0#coin_ref[Position(6465) := Dereference(__m, coin_ref)];
    assume is#Vector(check);
    __m := UpdateLocal(__m, __frame + 1, check);
    debug#LibraCoin#deposit#1#check := debug#LibraCoin#deposit#1#check[Position(6465) := check];

    // bytecode translation starts here
    call __t4 := CopyOrMoveRef(coin_ref);

    call __t5 := BorrowField(__t4, LibraCoin_T_value);

    call __tmp := ReadRef(__t5);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    debug#LibraCoin#deposit#2#value := debug#LibraCoin#deposit#2#value[Position(6729) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8 := Unpack_LibraCoin_T(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 8, __t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    debug#LibraCoin#deposit#3#check_value := debug#LibraCoin#deposit#3#check_value[Position(6779) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __t12 := CopyOrMoveRef(coin_ref);

    call __t13 := BorrowField(__t12, LibraCoin_T_value);

    call WriteRef(__t13, GetLocal(__m, __frame + 11));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraCoin_deposit_verify (coin_ref: Reference, check: Value) returns ()
{
    call InitVerification();
    call LibraCoin_deposit(coin_ref, check);
}

procedure {:inline 1} LibraCoin_destroy_zero (coin: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(!IsEqual(SelectField(coin, LibraCoin_T_value), Integer(0)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(!IsEqual(SelectField(coin, LibraCoin_T_value), Integer(0))))) ==> __abort_flag;

{
    // declare local variables
    var value: Value; // IntegerType()
    var __t2: Value; // LibraCoin_T_type_value()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // BooleanType()
    var __t7: Value; // BooleanType()
    var __t8: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#LibraCoin#destroy_zero#0#coin: [Position]Value;
    var debug#LibraCoin#destroy_zero#1#value: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 9;
    debug#LibraCoin#destroy_zero#0#coin := EmptyPositionMap;
    debug#LibraCoin#destroy_zero#1#value := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(coin);
    __m := UpdateLocal(__m, __frame + 0, coin);
    debug#LibraCoin#destroy_zero#0#coin := debug#LibraCoin#destroy_zero#0#coin[Position(7117) := coin];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t3 := Unpack_LibraCoin_T(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#LibraCoin#destroy_zero#1#value := debug#LibraCoin#destroy_zero#1#value[Position(7227) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5)));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __tmp := GetLocal(__m, __frame + 7);
    if (!b#Boolean(__tmp)) { goto Label_10; }

    call __tmp := LdConst(11);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    goto Label_Abort;

Label_10:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure LibraCoin_destroy_zero_verify (coin: Value) returns ()
{
    call InitVerification();
    call LibraCoin_destroy_zero(coin);
}
