// Copyright 2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use alloy_primitives::{Address, B256, U256};

use crate::{
    contract::read_return_data,
    hostio::{self, RETURN_DATA_SIZE},
};

/// Mechanism for performing raw deploys of other contracts.
#[derive(Clone, Default)]
#[must_use]
pub struct RawDeploy {
    salt: Option<B256>,
    offset: usize,
    size: Option<usize>,
}

impl RawDeploy {
    /// Begin configuring the raw deploy.
    pub fn new() -> Self {
        Default::default()
    }

    /// Configure the deploy to use the salt provided.
    /// This will use [`CREATE2`] under the hood to provide a deterministic address.
    ///
    /// [`CREATE2`]: https://www.evm.codes/#f5
    pub fn salt(mut self, salt: B256) -> Self {
        self.salt = Some(salt);
        self
    }

    /// Configure the deploy to use the salt provided.
    /// This will use [`CREATE2`] under the hood to provide a deterministic address if [`Some`].
    ///
    /// [`CREATE2`]: https://www.evm.codes/#f5
    pub fn salt_option(mut self, salt: Option<B256>) -> Self {
        self.salt = salt;
        self
    }

    /// Configures what portion of the revert data to copy in case of failure.
    /// Does not fail if out of bounds, but rather copies the overlapping portion.
    pub fn limit_revert_data(mut self, offset: usize, size: usize) -> Self {
        self.offset = offset;
        self.size = Some(size);
        self
    }

    /// Configures the call to avoid copying any revert data.
    /// Equivalent to `limit_revert_data(0, 0)`.
    pub fn skip_revert_data(self) -> Self {
        self.limit_revert_data(0, 0)
    }

    /// Performs a raw deploy of another contract with the given `endowment` and init `code`.
    /// Returns the address of the newly deployed contract, or the error data in case of failure.
    ///
    /// # Safety
    ///
    /// Note that the EVM allows init code to make calls to other contracts, which provides a vector force
    /// reentrancy. This means that this method may enable storage aliasing if used in the middle of a storage
    /// reference's lifetime and if reentrancy is allowed.
    ///
    /// For extra flexibility, this method does not clear the global storage cache.
    /// See [`StorageCache::flush`] and [`StorageCache::clear`] for more information.
    pub unsafe fn deploy(self, code: &[u8], endowment: U256) -> Result<Address, Vec<u8>> {
        let mut contract = Address::default();
        let mut revert_data_len = 0;

        let endowment: B256 = endowment.into();
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

        if contract.is_zero() {
            return Err(read_return_data(0, None));
        }
        Ok(contract)
    }
}
