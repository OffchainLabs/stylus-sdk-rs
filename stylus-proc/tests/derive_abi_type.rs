// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

// contract-client-gen feature can generate code that makes some imports and variables of this file unused
#![allow(unused_imports)]

extern crate alloc;

use alloy_sol_types::{sol, SolType};
use stylus_proc::AbiType;
use stylus_sdk::abi::AbiType;

macro_rules! test_abi_type {
    ($test_name:ident, $type:ty, $expected_abi:literal, $expected_selector:literal $(,)? ) => {
        #[test]
        // contract-client-gen disables derive abi type code generation
        #[cfg(not(feature = "contract-client-gen"))]
        fn $test_name() {
            assert_eq!(<$type as AbiType>::ABI.as_str(), $expected_abi);
            assert_eq!(
                <$type as AbiType>::SELECTOR_ABI.as_str(),
                $expected_selector
            );
            assert_eq!(
                <$type as AbiType>::ABI.as_str(),
                <$type as AbiType>::SolType::SOL_NAME,
            );
        }
    };
}

sol! {
    #[derive(Debug, PartialEq, AbiType)]
    struct SimpleStruct {
        uint8 bar;
    }

    #[derive(Debug, PartialEq, AbiType)]
    struct InnerStruct {
        address _address;
        string _string;
    }

    #[derive(Debug, PartialEq, AbiType)]
    struct NestedStruct {
        InnerStruct nested;
    }

    #[derive(Debug, PartialEq, AbiType)]
    struct ComplexStruct {
        address[] addrs;
        uint8[3] ints;
        bytes _bytes;
        (uint8, uint8) _tuple;
    }
}

test_abi_type!(simple_struct, SimpleStruct, "SimpleStruct", "(uint8)");
test_abi_type!(inner_struct, InnerStruct, "InnerStruct", "(address,string)");
test_abi_type!(
    nested_struct,
    NestedStruct,
    "NestedStruct",
    "((address,string))"
);
test_abi_type!(
    complex_struct,
    ComplexStruct,
    "ComplexStruct",
    "(address[],uint8[3],bytes,(uint8,uint8))"
);
test_abi_type!(
    struct_inside_vec,
    alloc::vec::Vec<SimpleStruct>,
    "SimpleStruct[]",
    "(uint8)[]",
);
test_abi_type!(
    struct_inside_array,
    [SimpleStruct; 3],
    "SimpleStruct[3]",
    "(uint8)[3]",
);
test_abi_type!(
    struct_inside_tuple,
    (SimpleStruct, ComplexStruct),
    "(SimpleStruct,ComplexStruct)",
    "((uint8),(address[],uint8[3],bytes,(uint8,uint8)))",
);

#[test]
fn encode() {
    let mut expected = [0u8; 32];
    expected[31] = 100;
    assert_eq!(
        <SimpleStruct as SolType>::abi_encode(&SimpleStruct { bar: 100 }),
        expected,
    );
}

#[test]
fn decode() {
    let mut input = [0u8; 32];
    input[31] = 100;
    assert_eq!(
        <SimpleStruct as SolType>::abi_decode(&input),
        Ok(SimpleStruct { bar: 100 }),
    );
}

#[cfg(all(not(feature = "contract-client-gen"), feature = "trybuild-tests"))]
#[test]
fn abi_type_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/derive_abi_type/missing_sol_macro.rs");
}
