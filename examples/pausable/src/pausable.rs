use stylus_sdk::{
    evm, msg,
    prelude::*,
    alloy_sol_types::{sol, SolError},
};

// Declare events and error types
sol! {
    event Paused(address account);
    event Unpaused(address account);

    error EnforcedPause();
    error ExpectedPause();
}

sol_storage! {
    pub struct Pausable {
        /// Indicates whether the contract is paused
        bool paused;
    }
}

pub enum PausableError {
    EnforcedPause(EnforcedPause),
    ExpectedPause(ExpectedPause),
}

// There will soon be a better way to deal with Custom errors, but for now this is the best way
impl From<PausableError> for Vec<u8> {
    fn from(err: PausableError) -> Vec<u8> {
        match err {
            PausableError::EnforcedPause(e) => e.encode(),
            PausableError::ExpectedPause(e) => e.encode(),
        }
    }
}

// Internal methods
impl Pausable {

    pub fn when_not_paused(&self) -> Result<(), PausableError>{
        if self.paused.get() {
            return Err(PausableError::EnforcedPause(EnforcedPause {}));
        }
        Ok(())
    }

    pub fn when_paused(&self) -> Result<(), PausableError>{
        if !self.paused.get() {
            return Err(PausableError::ExpectedPause(ExpectedPause {}));
        }
        Ok(())
    }

    // Internal function to pause the contract
    pub fn pause(&mut self) -> Result<(), PausableError> {
        self.when_not_paused()?;
        self.paused.set(true);
        evm::log(Paused { account: msg::sender() });
        Ok(())
    }

    // Internal function to unpause the contract
    pub fn unpause(&mut self) -> Result<(), PausableError> {
        self.when_paused()?;
        self.paused.set(false);
        evm::log(Unpaused { account: msg::sender() });
        Ok(())
    }

}

// External methods
#[external]
impl Pausable {
    // Check if the contract is paused; for external callers
    pub fn paused(&self) -> Result<(bool), PausableError> {
        Ok(self.paused.get())
    }
}