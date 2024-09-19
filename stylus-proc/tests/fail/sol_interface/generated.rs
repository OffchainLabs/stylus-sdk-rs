// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Compilation failures after macro generation completes.

extern crate alloc;

use stylus_proc::sol_interface;

sol_interface! {
    interface IService {
        #[function_attr]
        function makePayment(address user) payable external returns (string);
        function getConstant() pure external returns (bytes32);
    }

    #[interface_attr]
    interface ITree {
        // Define more interface methods here
    }
}

fn main() {}
