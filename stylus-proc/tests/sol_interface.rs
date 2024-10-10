// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Integration test for the `sol_interface!` macro

use stylus_proc::sol_interface;

mod inner {
    use alloy_sol_types::sol;

    sol! {
        struct Foo {
            uint256 bar;
        }
    }
}

sol_interface! {
    #[derive(Debug)]
    interface IService {
        function makePayment(address user) payable external returns (string);
        function getConstant() pure external returns (bytes32);
        function getFoo() pure external returns (inner.Foo);
    }

    interface ITree {
        // Define more interface methods here
    }
}

#[test]
fn sol_interface_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/sol_interface/macro_errors.rs");
    t.compile_fail("tests/fail/sol_interface/generated.rs");
}
