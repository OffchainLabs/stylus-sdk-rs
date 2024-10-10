// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Compilation should fail for any unsupported Solidity features.

use stylus_proc::sol_interface;

sol_interface! {
    #![file_attribute]

    interface IParent {
        function makePayment(address user) payable external returns (string);
        function getConstant() pure external returns (bytes32);
    }

    interface IChild is IParent {
    }

    contract TestContract {
    }

    function sum(uint[] memory arr) pure returns (uint s) {
        for (uint i = 0; i < arr.length; i++)
            s += arr[i];
    }
}

fn main() {}
