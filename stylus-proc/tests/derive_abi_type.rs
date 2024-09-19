// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use stylus_proc::AbiType;
use stylus_sdk::abi::AbiType;
use stylus_sdk::alloy_sol_types::sol;
use stylus_sdk::alloy_sol_types::SolType;

sol! {
    #[derive(Debug, PartialEq, AbiType)]
    struct MyStruct {
        uint8 bar;
    }
}

#[test]
fn test_abi_type() {
    assert_eq!(<MyStruct as AbiType>::ABI.as_str(), "MyStruct");
    assert_eq!(
        <MyStruct as AbiType>::ABI.as_str(),
        <MyStruct as SolType>::SOL_NAME,
    );
}

#[test]
fn test_abi_encode() {
    let mut expected = [0u8; 32];
    expected[31] = 100;
    assert_eq!(
        <MyStruct as SolType>::abi_encode(&MyStruct { bar: 100 }),
        expected,
    );
}

#[test]
fn test_abi_decode() {
    let mut input = [0u8; 32];
    input[31] = 100;
    assert_eq!(
        <MyStruct as SolType>::abi_decode(&input, true),
        Ok(MyStruct { bar: 100 }),
    );
}

#[test]
fn test_derive_abi_type_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/derive_abi_type/missing_sol_macro.rs");
}
