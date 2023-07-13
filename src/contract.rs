// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use crate::{
    hostio::{self, wrap_hostio},
    types::AddressVM,
};
use alloy_primitives::{Address, B256};

#[derive(Clone, Default)]
#[must_use]
pub struct Call {
    kind: CallKind,
    value: B256,
    ink: Option<u64>,
}

#[derive(Clone, Default, PartialEq)]
enum CallKind {
    #[default]
    Basic,
    Delegate,
    Static,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct RustVec {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}

impl Default for RustVec {
    fn default() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }
}

impl Call {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_delegate() -> Self {
        Self {
            kind: CallKind::Delegate,
            ..Default::default()
        }
    }

    pub fn new_static() -> Self {
        Self {
            kind: CallKind::Static,
            ..Default::default()
        }
    }

    pub fn value(mut self, value: B256) -> Self {
        if self.kind != CallKind::Basic {
            panic!("cannot set value for delegate or static calls");
        }
        self.value = value;
        self
    }

    pub fn ink(mut self, ink: u64) -> Self {
        self.ink = Some(ink);
        self
    }

    pub fn call(self, contract: Address, calldata: &[u8]) -> Result<Vec<u8>, Vec<u8>> {
        let mut outs_len = 0;
        let ink = self.ink.unwrap_or(u64::MAX); // will be clamped by 63/64 rule
        let status = unsafe {
            match self.kind {
                CallKind::Basic => hostio::call_contract(
                    contract.as_ptr(),
                    calldata.as_ptr(),
                    calldata.len(),
                    self.value.as_ptr(),
                    ink,
                    &mut outs_len,
                ),
                CallKind::Delegate => hostio::delegate_call_contract(
                    contract.as_ptr(),
                    calldata.as_ptr(),
                    calldata.len(),
                    ink,
                    &mut outs_len,
                ),
                CallKind::Static => hostio::static_call_contract(
                    contract.as_ptr(),
                    calldata.as_ptr(),
                    calldata.len(),
                    ink,
                    &mut outs_len,
                ),
            }
        };

        let mut outs = Vec::with_capacity(outs_len);
        if outs_len != 0 {
            unsafe {
                hostio::read_return_data(outs.as_mut_ptr());
                outs.set_len(outs_len);
            }
        };
        match status {
            0 => Ok(outs),
            _ => Err(outs),
        }
    }
}

pub fn create(code: &[u8], endowment: B256, salt: Option<B256>) -> Result<Address, Vec<u8>> {
    let mut contract = [0; 20];
    let mut revert_data_len = 0;
    let contract = unsafe {
        if let Some(salt) = salt {
            hostio::create2(
                code.as_ptr(),
                code.len(),
                endowment.as_ptr(),
                salt.as_ptr(),
                contract.as_mut_ptr(),
                &mut revert_data_len as *mut _,
            );
        } else {
            hostio::create1(
                code.as_ptr(),
                code.len(),
                endowment.as_ptr(),
                contract.as_mut_ptr(),
                &mut revert_data_len as *mut _,
            );
        }
        Address::from(contract)
    };
    if contract.is_zero() {
        unsafe {
            let mut revert_data = Vec::with_capacity(revert_data_len);
            hostio::read_return_data(revert_data.as_mut_ptr());
            revert_data.set_len(revert_data_len);
            return Err(revert_data);
        }
    }
    Ok(contract)
}

wrap_hostio!(
    /// Returns the length of the last EVM call or deployment return result, or `0` if neither have
    /// happened during the program's execution.
    return_data_len return_data_size usize
);

wrap_hostio!(
    /// Gets the address of the current program.
    address contract_address Address
);

/// Gets the balance of the current program.
pub fn balance() -> B256 {
    address().balance()
}
