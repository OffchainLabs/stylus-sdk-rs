// Copyright 2023, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro/blob/master/LICENSE

use crate::{
    hostio::{self, wrap_hostio, RETURN_DATA_SIZE},
    tx,
    types::AddressVM,
};
use alloy_primitives::{Address, B256, U64};

#[derive(Clone, Default)]
#[must_use]
pub struct Call {
    kind: CallKind,
    value: B256,
    gas: Option<u64>,
    offset: usize,
    size: Option<usize>,
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

    pub fn value(mut self, callvalue: B256) -> Self {
        if self.kind != CallKind::Basic {
            panic!("cannot set value for delegate or static calls");
        }
        self.value = callvalue;
        self
    }

    pub fn gas(mut self, gas: U64) -> Self {
        self.gas = Some(gas.try_into().unwrap());
        self
    }

    pub fn ink(mut self, ink: U64) -> Self {
        self.gas = Some(tx::ink_to_gas(ink).try_into().unwrap());
        self
    }

    pub fn limit_return_data(mut self, offset: usize, size: usize) -> Self {
        self.offset = offset;
        self.size = Some(size);
        self
    }

    pub fn skip_return_data(self) -> Self {
        self.limit_return_data(0, 0)
    }

    pub fn call(self, contract: Address, calldata: &[u8]) -> Result<Vec<u8>, Vec<u8>> {
        let mut outs_len = 0;
        let gas = self.gas.unwrap_or(u64::MAX); // will be clamped by 63/64 rule
        let status = unsafe {
            match self.kind {
                CallKind::Basic => hostio::call_contract(
                    contract.as_ptr(),
                    calldata.as_ptr(),
                    calldata.len(),
                    self.value.as_ptr(),
                    gas,
                    &mut outs_len,
                ),
                CallKind::Delegate => hostio::delegate_call_contract(
                    contract.as_ptr(),
                    calldata.as_ptr(),
                    calldata.len(),
                    gas,
                    &mut outs_len,
                ),
                CallKind::Static => hostio::static_call_contract(
                    contract.as_ptr(),
                    calldata.as_ptr(),
                    calldata.len(),
                    gas,
                    &mut outs_len,
                ),
            }
        };

        unsafe {
            RETURN_DATA_SIZE.set(outs_len);
        }

        let outs = read_return_data(self.offset, self.size);
        match status {
            0 => Ok(outs),
            _ => Err(outs),
        }
    }
}

#[derive(Clone, Default)]
#[must_use]
pub struct Deploy {
    salt: Option<B256>,
    offset: usize,
    size: Option<usize>,
}

impl Deploy {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn salt(mut self, salt: Bytes32) -> Self {
        self.salt = Some(salt);
        self
    }

    pub fn salt_option(mut self, salt: Option<Bytes32>) -> Self {
        self.salt = salt;
        self
    }

    pub fn limit_return_data(mut self, offset: usize, size: usize) -> Self {
        self.offset = offset;
        self.size = Some(size);
        self
    }

    pub fn skip_return_data(self) -> Self {
        self.limit_return_data(0, 0)
    }

    pub fn deploy(self, code: &[u8], endowment: Bytes32) -> Result<Bytes20, Vec<u8>> {
        let mut contract = Bytes20::default();
        let mut revert_data_len = 0;
        unsafe {
            if let Some(salt) = self.salt {
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
            RETURN_DATA_SIZE.set(revert_data_len);
        }
        if contract.is_zero() {
            return Err(read_return_data(0, None));
        }
        Ok(contract)
    }
}

pub fn read_return_data(offset: usize, size: Option<usize>) -> Vec<u8> {
    let size = unsafe { size.unwrap_or_else(|| RETURN_DATA_SIZE.get().saturating_sub(offset)) };

    let mut data = Vec::with_capacity(size);
    if size > 0 {
        unsafe {
            let bytes_written = hostio::read_return_data(data.as_mut_ptr(), offset, size);
            debug_assert!(bytes_written <= size);
            data.set_len(bytes_written);
        }
    };
    data
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
