#![cfg_attr(not(feature = "export-abi"), no_std, no_main)]
extern crate alloc;


use stylus_sdk::{alloy_primitives::{Address, U256, FixedBytes}, prelude::*};
use alloc::{vec, vec::Vec};

sol_storage! {
    #[entrypoint]
    pub struct Complex {
        mapping(address => uint256) balances;
        mapping(uint256 => address) owners;
        address[] registered_users;
        uint256 total_supply;
        bytes32 merkle_root;
    }
}

#[public]
impl Complex {
    pub fn register_user(&mut self, user: Address) {
        self.registered_users.push(user);
        self.balances.setter(user).set(U256::from(100));
        self.total_supply.set(self.total_supply.get() + U256::from(100));
    }

    pub fn transfer(&mut self, from: Address, to: Address, amount: U256) -> Result<(), Vec<u8>> {
        let from_balance = self.balances.get(from);
        if from_balance < amount {
            return Err(b"Insufficient balance".to_vec());
        }
        
        self.balances.setter(from).set(from_balance - amount);
        let to_balance = self.balances.get(to);
        self.balances.setter(to).set(to_balance + amount);
        
        Ok(())
    }

    pub fn nested_computation(&mut self, depth: u32) -> U256 {
        if depth == 0 {
            return U256::from(1);
        }
        
        let result = self.nested_computation(depth - 1);
        result * U256::from(2)
    }

    pub fn complex_loop(&mut self, iterations: U256) -> U256 {
        let mut result = U256::ZERO;
        let mut temp = U256::from(1);
        
        for i in 0..iterations.to::<u32>() {
            temp = temp * U256::from(2);
            result = result + temp;
            
            // Add some storage operations
            if i % 10 == 0 {
                let sender = self.vm().msg_sender();
                self.owners.setter(U256::from(i)).set(sender);
            }
        }
        
        result
    }

    pub fn process_batch(&mut self, users: Vec<Address>, amounts: Vec<U256>) -> Result<U256, Vec<u8>> {
        if users.len() != amounts.len() {
            return Err(b"Length mismatch".to_vec());
        }
        
        let mut total = U256::ZERO;
        for (i, user) in users.iter().enumerate() {
            let amount = amounts[i];
            let current = self.balances.get(*user);
            self.balances.setter(*user).set(current + amount);
            total = total + amount;
        }
        
        self.total_supply.set(self.total_supply.get() + total);
        Ok(total)
    }

    pub fn update_merkle_root(&mut self, new_root: FixedBytes<32>) {
        self.merkle_root.set(new_root);
    }

    pub fn verify_and_claim(&mut self, user: Address, amount: U256, proof: Vec<FixedBytes<32>>) -> Result<(), Vec<u8>> {
        // Simulate merkle proof verification
        let mut computed_hash = keccak256(&[user.as_slice(), &amount.to_be_bytes::<32>()].concat());
        
        for proof_element in proof.iter() {
            if computed_hash < *proof_element {
                computed_hash = keccak256(&[computed_hash.as_slice(), proof_element.as_slice()].concat());
            } else {
                computed_hash = keccak256(&[proof_element.as_slice(), computed_hash.as_slice()].concat());
            }
        }
        
        if computed_hash != self.merkle_root.get() {
            return Err(b"Invalid proof".to_vec());
        }
        
        // Process claim
        let current = self.balances.get(user);
        self.balances.setter(user).set(current + amount);
        self.total_supply.set(self.total_supply.get() + amount);
        
        Ok(())
    }

    pub fn get_balance(&self, user: Address) -> U256 {
        self.balances.get(user)
    }

    pub fn get_total_supply(&self) -> U256 {
        self.total_supply.get()
    }

    pub fn get_registered_users_count(&self) -> U256 {
        U256::from(self.registered_users.len())
    }
}

// Helper function for keccak256
fn keccak256(data: &[u8]) -> FixedBytes<32> {
    use stylus_sdk::crypto::keccak;
    keccak(data)
}

