//! Native (non-wasm32) test verifying `sol_interface!` calls route through native hostio stubs
//! and can be mocked via `hostio::testing::mock_static_call`.
#![cfg(all(not(target_arch = "wasm32"), feature = "hostio"))]

// Required because the generated `sol_interface!` code references the `alloc` crate explicitly
extern crate alloc;

use alloy_primitives::Address;
use stylus_sdk::hostio; // only public when feature = hostio
use stylus_sdk::prelude::*;
use stylus_sdk::storage::TopLevelStorage;

// Minimal top-level storage context for static calls in tests.
struct Dummy;
unsafe impl TopLevelStorage for Dummy {}

sol_interface! {
    interface IAdder {
        function add(uint64 a, uint64 b) external view returns (uint128);
    }
}

fn encode_add(a: u64, b: u64) -> Vec<u8> {
    // selector keccak("add(uint64,uint64)")[0..4] = 6e2c732d
    let mut data = vec![0x6e, 0x2c, 0x73, 0x2d];
    data.extend([0u8; 24]);
    data.extend(a.to_be_bytes());
    data.extend([0u8; 24]);
    data.extend(b.to_be_bytes());
    data
}

#[test]
fn sol_interface_static_call_mock() {
    // 0x...01 test address
    let target: Address = {
        let mut b = [0u8; 20];
        b[19] = 1;
        Address::from(b)
    };
    // Expect add(10,5) -> 15u128 encoded as 32-byte big-endian word
    let ret = 15u128.to_be_bytes();
    let mut encoded = vec![0u8; 16]; // pad to 32 bytes
    encoded.extend_from_slice(&ret);

    hostio::testing::mock_static_call(target, encode_add(10, 5), Ok(encoded.clone()));

    let iface = IAdder::new(target);
    let ctx = Dummy; // &ctx implements StaticCallContext via TopLevelStorage blanket impls
    let result = iface.add(&ctx, 10, 5).expect("mocked ok");
    assert_eq!(result, 15u128);
}
