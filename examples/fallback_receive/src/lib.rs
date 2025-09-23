// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use crate::alloc::string::ToString;
use crate::vec::Vec;
use alloc::vec;
use stylus_sdk::alloy_sol_types::sol;
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256},
    evm::log,
    prelude::*,
    ArbResult,
};
// Define persistent storage
sol_storage! {
    #[entrypoint]
    pub struct PaymentTracker {
        uint256 total_received;
        uint256 fallback_calls;
        uint256 receive_calls;
        mapping(address => uint256) balances;
    }
}

// Define events for better tracking
sol! {
    event EtherReceived(address indexed sender, uint256 amount, string method);
    event FallbackTriggered(address indexed sender, uint256 amount, bytes data);
    event UnknownFunctionCalled(address indexed sender, bytes4 selector);
}

#[public]
impl PaymentTracker {
    // Regular function to check balance
    pub fn get_balance(&self, account: Address) -> U256 {
        self.balances.get(account)
    }

    // Regular function to get statistics
    pub fn get_stats(&self) -> (U256, U256, U256) {
        (
            self.total_received.get(),
            self.receive_calls.get(),
            self.fallback_calls.get(),
        )
    }

    /// Receive function - handles plain Ether transfers
    /// This is called when someone sends Ether without any data
    /// Example: contract.send(1 ether) or contract.transfer(1 ether)
    #[receive]
    #[payable]
    pub fn receive(&mut self) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        let amount = self.vm().msg_value();

        // Update tracking variables
        self.total_received.set(self.total_received.get() + amount);
        self.receive_calls
            .set(self.receive_calls.get() + U256::from(1));

        // Update sender's balance using setter method
        let current_balance = self.balances.get(sender);
        self.balances.setter(sender).set(current_balance + amount);

        // Log the event
        self.vm().log(EtherReceived {
            sender,
            amount,
            method: "receive".to_string(),
        });

        Ok(())
    }

    /// Fallback function - handles unmatched function calls
    /// This is called when:
    /// 1. A function call doesn't match any existing function signature
    /// 2. Plain Ether transfer when no receive function exists
    #[fallback]
    #[payable]
    pub fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        let sender = self.vm().msg_sender();
        let amount = self.vm().msg_value();

        // Update tracking
        self.fallback_calls
            .set(self.fallback_calls.get() + U256::from(1));

        if amount > U256::ZERO {
            // If Ether was sent, track it
            self.total_received.set(self.total_received.get() + amount);
            let current_balance = self.balances.get(sender);
            self.balances.setter(sender).set(current_balance + amount);

            self.vm().log(EtherReceived {
                sender,
                amount,
                method: "fallback".to_string(),
            });
        }

        // Log the fallback trigger with calldata - convert to bytes properly
        self.vm().log(FallbackTriggered {
            sender,
            amount,
            data: calldata.to_vec().into(),
        });

        // If calldata has at least 4 bytes, extract the function selector
        if calldata.len() >= 4 {
            let selector = [calldata[0], calldata[1], calldata[2], calldata[3]];
            self.vm().log(UnknownFunctionCalled {
                sender,
                selector: FixedBytes(selector),
            });
        }

        // Return empty bytes (successful execution)
        Ok(vec![])
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stylus_sdk::alloy_primitives::{keccak256, B256};
    use stylus_sdk::testing::*;

    #[test]
    fn test_receive_function() {
        let vm = TestVM::default();
        let mut contract = PaymentTracker::from(&vm);

        // Test that contract is created successfully and initial values are correct
        let (total, receive_calls, fallback_calls) = contract.get_stats();
        assert_eq!(total, U256::from(0));
        assert_eq!(receive_calls, U256::from(0));
        assert_eq!(fallback_calls, U256::from(0));
        // Override the msg value for future contract method invocations.
        vm.set_value(U256::from(2));
        let _ = contract.receive();
        // Check that the receive function updates stats correctly
        let (total, receive_calls, fallback_calls) = contract.get_stats();
        assert_eq!(total, U256::from(2));
        assert_eq!(receive_calls, U256::from(1));
        assert_eq!(fallback_calls, U256::from(0));
        // Check that the balance is updated
        let balance = contract.balances.get(vm.msg_sender());
        assert_eq!(balance, U256::from(2));
    }

    #[test]
    fn test_fallback_function() {
        let vm = TestVM::default();
        let mut contract = PaymentTracker::from(&vm);

        // Test that contract is created successfully and initial values are correct
        let (total, receive_calls, fallback_calls) = contract.get_stats();
        assert_eq!(total, U256::from(0));
        assert_eq!(receive_calls, U256::from(0));
        assert_eq!(fallback_calls, U256::from(0));
        // Call the fallback function with some data
        let calldata = vec![0x01, 0x02, 0x03, 0x04];
        let _ = contract.fallback(&calldata);
        // Check that the fallback function updates stats correctly
        let (total, receive_calls, fallback_calls) = contract.get_stats();
        assert_eq!(total, U256::from(0));
        assert_eq!(receive_calls, U256::from(0));
        assert_eq!(fallback_calls, U256::from(1));
        // Check that the balance is updated
        let balance = contract.balances.get(vm.msg_sender());
        assert_eq!(balance, U256::from(0));

        // Check that the fallback triggered event was logged
        let logs = vm.get_emitted_logs();
        assert_eq!(logs.len(), 2);

        // Check that the first log is the FallbackTriggered event
        let event_signature = B256::from(keccak256(
            "FallbackTriggered(address,uint256,bytes)".as_bytes(),
        ));
        assert_eq!(logs[0].0[0], event_signature);
        // Check that the second log is the UnknownFunctionCalled event
        let unknown_signature = B256::from(keccak256(
            "UnknownFunctionCalled(address,bytes4)".as_bytes(),
        ));
        assert_eq!(logs[1].0[0], unknown_signature);
    }

    #[test]
    fn test_fallback_function_with_value() {
        let vm = TestVM::default();
        let mut contract = PaymentTracker::from(&vm);

        // Test that contract is created successfully and initial values are correct
        let (total, receive_calls, fallback_calls) = contract.get_stats();
        assert_eq!(total, U256::from(0));
        assert_eq!(receive_calls, U256::from(0));
        assert_eq!(fallback_calls, U256::from(0));

        vm.set_value(U256::from(2));
        let calldata = vec![0x01, 0x02, 0x03, 0x04];
        // Call the fallback function with calldata and value
        let _ = contract.fallback(&calldata);
        // Check that the fallback function updates stats correctly
        let (total, receive_calls, fallback_calls) = contract.get_stats();
        assert_eq!(total, U256::from(2));
        assert_eq!(receive_calls, U256::from(0));
        assert_eq!(fallback_calls, U256::from(1));
        // Check that the balance is updated
        let balance = contract.balances.get(vm.msg_sender());
        assert_eq!(balance, U256::from(2));
        // Check that the fallback triggered event was logged
        let logs = vm.get_emitted_logs();
        assert_eq!(logs.len(), 3);
        // Check that the first log is the FallbackTriggered event
        let event_signature = B256::from(keccak256(
            "EtherReceived(address,uint256,string)".as_bytes(),
        ));
        assert_eq!(logs[0].0[0], event_signature);
        // Check that the second log is the EtherReceived event
        let ether_received_signature = B256::from(keccak256(
            "FallbackTriggered(address,uint256,bytes)".as_bytes(),
        ));
        assert_eq!(logs[1].0[0], ether_received_signature);
        // Check that the third log is the UnknownFunctionCalled event
        let unknown_signature = B256::from(keccak256(
            "UnknownFunctionCalled(address,bytes4)".as_bytes(),
        ));
        assert_eq!(logs[2].0[0], unknown_signature);
    }
}
