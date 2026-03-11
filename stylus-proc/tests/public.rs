// Copyright 2024-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Tests for the `#[public]` macro, including successful compilation of valid contracts
//! (with overloads, near-miss names, and resolved collisions) and compile-fail tests for
//! selector collisions.

#![allow(dead_code)]
// contract-client-gen feature can generate code that makes some imports of this file unused
#![allow(unused_imports)]

extern crate alloc;

use alloy_primitives::U256;
use stylus_proc::public;
use stylus_sdk::{prelude::*, storage::StorageU256, ArbResult};

#[storage]
#[entrypoint]
pub struct Contract {
    value: StorageU256,
}

#[public]
impl Contract {
    #[payable]
    fn method() {}

    #[fallback]
    fn fallback(&mut self, _args: &[u8]) -> ArbResult {
        Ok(vec![])
    }

    #[receive]
    fn receive(&mut self) -> Result<(), Vec<u8>> {
        Ok(())
    }

    #[constructor]
    fn constructor(&mut self, value: U256) {
        self.value.set(value);
    }

    fn value(&self) -> Result<U256, Vec<u8>> {
        Ok(self.value.get())
    }

    fn another_method(&self) -> Result<bool, Vec<u8>> {
        Ok(true)
    }

    fn yet_another(&mut self) -> Result<(), Vec<u8>> {
        Ok(())
    }

    // Near-miss camelCase names: foo_bar_baz -> fooBarBaz, foo_barbaz -> fooBarbaz
    // These should NOT collide despite similar names.
    fn foo_bar_baz(&self) -> Result<bool, Vec<u8>> {
        Ok(true)
    }

    fn foo_barbaz(&self) -> Result<bool, Vec<u8>> {
        Ok(false)
    }

    // Same Solidity name "transfer" but different parameter lists: the ABI selector includes
    // parameter types in the keccak hash, so transfer(uint256) and transfer(uint256,uint256)
    // naturally produce different 4-byte selectors and do not collide.
    #[selector(name = "transfer")]
    fn transfer_one(&self, _value: U256) -> Result<(), Vec<u8>> {
        Ok(())
    }

    #[selector(name = "transfer")]
    fn transfer_two(&self, _from: U256, _value: U256) -> Result<(), Vec<u8>> {
        Ok(())
    }

    // Same Solidity name "convert" but different parameter types: convert(uint256) vs
    // convert(bool) produce different selectors because the full signature is hashed.
    #[selector(name = "convert")]
    fn convert_u256(&self, _x: U256) -> Result<(), Vec<u8>> {
        Ok(())
    }

    #[selector(name = "convert")]
    fn convert_bool(&self, _x: bool) -> Result<(), Vec<u8>> {
        Ok(())
    }
}

// Fallback/receive/constructor are excluded from collision detection.
#[storage]
pub struct SpecialFnContract {}

#[public]
impl SpecialFnContract {
    fn regular(&self) -> Result<bool, Vec<u8>> {
        Ok(true)
    }

    #[fallback]
    fn fallback(&mut self, _args: &[u8]) -> ArbResult {
        Ok(vec![])
    }

    #[receive]
    fn receive(&mut self) -> Result<(), Vec<u8>> {
        Ok(())
    }

    #[constructor]
    fn constructor(&mut self, _value: U256) {}
}

// Collision resolved via #[selector(name = "...")]: foo_bar and fooBar would both become
// "fooBar" in Solidity (causing a collision), but #[selector(name = "fooBarAlt")] overrides
// one function's ABI name to avoid it.
#[storage]
pub struct ResolvedCollisionContract {}

#[public]
#[allow(non_snake_case)]
impl ResolvedCollisionContract {
    fn foo_bar(&self, _x: u64) -> Result<(), Vec<u8>> {
        Ok(())
    }

    #[selector(name = "fooBarAlt")]
    fn fooBar(&self, _x: u64) -> Result<(), Vec<u8>> {
        Ok(())
    }
}

#[cfg(all(not(feature = "contract-client-gen"), feature = "trybuild-tests"))]
#[test]
fn test_public_failures() {
    let t = trybuild::TestCases::new();
    #[cfg(not(feature = "export-abi"))]
    t.compile_fail("tests/fail/public/generated.rs");
    t.compile_fail("tests/fail/public/macro_errors.rs");
    t.compile_fail("tests/fail/public/constructor.rs");
    t.compile_fail("tests/fail/public/missing_macros.rs");
    t.compile_fail("tests/fail/public/selector_collision_camelcase.rs");
    t.compile_fail("tests/fail/public/selector_collision_both_explicit.rs");
    t.compile_fail("tests/fail/public/selector_collision_multiple.rs");
    t.compile_fail("tests/fail/public/selector_collision_trait_only.rs");
    t.compile_fail("tests/fail/public/selector_collision_explicit_vs_auto.rs");
}
