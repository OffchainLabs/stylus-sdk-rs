use alloc::{vec::Vec};
use stylus_sdk::{
    alloy_primitives::{Address},
    alloy_sol_types::{sol, SolError},
    evm, msg,
    prelude::*,
    storage::StorageAddress,
};

#[solidity_storage]
#[entrypoint]
pub struct Ownable {
    owner: StorageAddress,
}

// Declare events and Solidity error types
sol! {
    event OwnershipTransferred(address indexed previous_owner, address indexed new_owner);

    error OwnableUnauthorizedAccount(address account);
    error OwnableInvalidOwner(address owner);
}

pub enum OwnableError {
    OwnableUnauthorizedAccount(OwnableUnauthorizedAccount),
    OwnableInvalidOwner(OwnableInvalidOwner),
}

// We will soon provide a #[derive(SolidityError)] to clean this up
impl From<OwnableError> for Vec<u8> {
    fn from(err: OwnableError) -> Vec<u8> {
        match err {
            OwnableError::OwnableUnauthorizedAccount(e) => e.encode(),
            OwnableError::OwnableInvalidOwner(e) => e.encode(),
        }
    }
}

// These methods aren't exposed to other contracts
// Note: modifying storage will become much prettier soon
impl Ownable {
    pub fn check_owner_impl(
        &mut self,
    ) -> Result<(), OwnableError> {
        if msg::sender() != self.owner.get() {
            return Err(OwnableError::OwnableUnauthorizedAccount(OwnableUnauthorizedAccount {
                account: msg::sender()
            }))
        }

        return Ok(())
    }

    pub fn transfer_ownership_impl(
        &mut self,
        new_owner: Address,
    ) {
        let old_owner  = self.owner.get();
        self.owner.set(new_owner);
        evm::log(OwnershipTransferred {
            previous_owner: old_owner,
            new_owner: new_owner
        });
    }
}

// These methods are external to other contracts
#[external]
impl Ownable {
    pub fn renounce_ownership(&mut self) -> Result<bool, OwnableError> {
        self.check_owner_impl()?;

        let zero_addr_str = "0x0000000000000000000000000000000000000000";
        let zero_address : Address = Address::parse_checksummed(zero_addr_str, None).unwrap();
        self.transfer_ownership_impl(zero_address);
        Ok(true)
    }

    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<bool, OwnableError> {
        self.check_owner_impl()?;
        self.transfer_ownership_impl(new_owner);
        Ok(true)
    }

    pub fn owner(&mut self) -> Result<Address, OwnableError> {
        Ok(self.owner.get())
    }
}
