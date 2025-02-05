use alloc::vec::Vec;
use alloy_primitives::{Address, B256, U256};
use stylus_core::deploy::DeploymentAccess;
use stylus_core::host::{CalldataAccess, UnsafeDeploymentAccess};

use super::WasmVM;

impl DeploymentAccess for WasmVM {
    #[cfg(feature = "reentrant")]
    unsafe fn deploy(
        &self,
        code: &[u8],
        endowment: U256,
        salt: Option<B256>,
        cache_policy: stylus_core::deploy::CachePolicy,
    ) -> Result<Address, Vec<u8>> {
        use stylus_core::deploy::CachePolicy;
        use stylus_core::host::StorageAccess;
        match cache_policy {
            CachePolicy::Clear => self.flush_cache(true),
            CachePolicy::Flush => self.flush_cache(false),
            CachePolicy::DoNothing => {}
        }

        let mut contract = Address::default();
        let mut revert_data_len: usize = 0;

        let endowment: B256 = endowment.into();
        if let Some(salt) = salt {
            self.create2(
                code.as_ptr(),
                code.len(),
                endowment.as_ptr(),
                salt.as_ptr(),
                contract.as_mut_ptr(),
                &mut revert_data_len as *mut _,
            );
        } else {
            self.create1(
                code.as_ptr(),
                code.len(),
                endowment.as_ptr(),
                contract.as_mut_ptr(),
                &mut revert_data_len as *mut _,
            );
        }
        if contract.is_zero() {
            return Err(self.read_return_data(0, None));
        }
        Ok(contract)
    }
    #[cfg(not(feature = "reentrant"))]
    unsafe fn deploy(
        &self,
        code: &[u8],
        endowment: U256,
        salt: Option<B256>,
    ) -> Result<Address, Vec<u8>> {
        let mut contract = Address::default();
        let mut revert_data_len: usize = 0;

        let endowment: B256 = endowment.into();
        if let Some(salt) = salt {
            self.create2(
                code.as_ptr(),
                code.len(),
                endowment.as_ptr(),
                salt.as_ptr(),
                contract.as_mut_ptr(),
                &mut revert_data_len as *mut _,
            );
        } else {
            self.create1(
                code.as_ptr(),
                code.len(),
                endowment.as_ptr(),
                contract.as_mut_ptr(),
                &mut revert_data_len as *mut _,
            );
        }
        if contract.is_zero() {
            return Err(self.read_return_data(0, None));
        }
        Ok(contract)
    }
}
