# Stylus Test

The stylus-test crate makes it easy to unit test all the storage types and contracts that use the Stylus SDK. Included is an implementation of the [stylus_core::host::Host](../stylus-core/latest/stylus_core/host/trait.Host.html) trait that all Stylus contracts have access to for interfacing with their host environment.

The mock implementation, named `TestVM`, can be used to unit test Stylus contracts in native Rust without the need for a real EVM or Arbitrum chain environment. The TestVM allows for mocking of all host functions, including storage, gas, and external calls to assert contract behavior.

To be able to unit test Stylus contracts, contracts must access host methods through the [HostAccess](https://docs.rs/stylus-core/latest/stylus_core/host/trait.HostAccess.html) trait, which gives all contracts access to a `.vm()` method. That is, instead of calling `stylus_sdk::msg::value()` directly, contracts should do `self.vm().msg_value()`.

## Getting Started

The stylus-test crate is **not meant to be used directly**, as it is already exported by the Stylus SDK. It can be accessed via `stylus_sdk::testing::*`. Here is how to use it for a basic `Counter` smart contract defined in the [stylus-hello-world](https://github.com/OffchainLabs/stylus-hello-world) template:

```rs
use stylus_sdk::{alloy_primitives::U256, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct Counter {
        uint256 number;
    }
}

#[public]
impl Counter {
    pub fn number(&self) -> U256 {
        self.number.get()
    }
    pub fn increment(&mut self) {
        let number = self.number.get();
        self.set_number(number + U256::from(1));
    }
    #[payable]
    pub fn add_from_msg_value(&mut self) {
        let number = self.number.get();
        self.set_number(number + self.vm().msg_value());
    }
}
```

After defining the contract above using the Stylus SDK, we can define native Rust unit tests as are commonly written in Rust projects:
```rs
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_counter() {
        use stylus_sdk::testing::*;
        ...
    }
}
```

The `stylus_sdk::testing::*` import gives your tests access to a powerful `TestVM` struct which lets you mock almost any part
of your contracts' host environment, such as the the message value of a transaction, or even raw storage. Here's how it works:

```rs
// We define a default TestVM. Note that we can customize it via a TestVMBuilder
// we will discuss later in this README.
let vm = TestVM::default();

// You can then initialize your Counter Stylus contract from a VM reference, as all
// Stylus storage types implement the `From<VM>` trait.
let mut contract = Counter::from(&vm);
```

Next, we can perform some basic assertions.

```rs
// The TestVM handles the internals of contract storage, so normal
// storage interactions will work as expected in your test.
assert_eq!(U256::ZERO, contract.number());
contract.increment();
assert_eq!(U256::from(1), contract.number());
```

Next, we can mock the message value of the transaction, and assert some of our contract's logic.

```rs
// Override the msg value.
vm.set_value(U256::from(2));

contract.add_from_msg_value();

// Assert the value is as we expect.
assert_eq!(U256::from(3), contract.number());
```

## TestVM Custom Builder

The TestVM default is easy to use, but one can further initialize a TestVM from custom values, even
using an external RPC endpoint to fork storage reads.

`stylus_sdk::testing::TestVMBuilder` Allows for convenient customization of the contract's address, sender address, message value, and RPC
URL if state forking is desired. These values and more can still be customized if the builder is not used,
by instead invoking the corresponding method on the TestVM struct such as `vm.set_msg_value(value)`.

```rs
use stylus_test::{TestVM, TestVMBuilder};
use alloy_primitives::{address, Address, U256};

let vm: TestVM = TestVMBuilder::new()
    .sender(address!("dCE82b5f92C98F27F116F70491a487EFFDb6a2a9"))
    .contract_address(address!("DeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF"))
    .value(U256::from(1))
    .rpc_url("http://localhost:8547")
    .build();
```

## Inspecting Emitted Logs and Mocking Calls

Logs emitted can also be inspected by the TestVM pattern:

```rs
#[test]
fn test_logs() {
    let vm = TestVM::new();
    let topic1 = B256::from([1u8; 32]);
    let topic2 = B256::from([2u8; 32]);
    let data = vec![3, 4, 5];

    vm.raw_log(&[topic1, topic2], &data).unwrap();

    let logs = vm.get_emitted_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].0, vec![topic1, topic2]);
    assert_eq!(logs[0].1, data);
}
```

Calls can easily be mocked:

```rs
#[test]
fn test_mock_calls() {
    let vm = TestVM::new();
    let target = Address::from([2u8; 20]);
    let data = vec![1, 2, 3, 4];
    let expected_return = vec![5, 6, 7, 8];

    // Mock a regular call.
    vm.mock_call(target, data.clone(), Ok(expected_return.clone()));

    let ctx = stylus_core::calls::context::Call::new();
    let result = vm.call(&ctx, target, &data).unwrap();
    assert_eq!(result, expected_return);

    // Mock an error case.
    let error_data = vec![9, 9, 9];
    vm.mock_call(target, data.clone(), Err(error_data.clone()));

    match vm.call(&ctx, target, &data) {
        Err(Error::Revert(returned_data)) => assert_eq!(returned_data, error_data),
        _ => panic!("Expected revert error"),
    }
}
```

## Writing your Own Custom TestVM

A TestVM is a simple struct implemented in the stylus-test crate that implements the `Host` trait from [stylus_core::host::Host](https://docs.rs/stylus-core/latest/stylus_core/host/trait.Host.html). Anyone can implement the trait and allow for rich testing experiences
for Stylus contracts. The TestVM is not the only way to unit test your projects, as you can extend or implement your own.