// Copyright 2024-2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Defines host environment methods Stylus SDK contracts have access to.
use alloc::vec::Vec;
use alloy_primitives::{Address, B256, U256};

/// The `wasm` module contains the default implementation of the host trait for all programs
/// that are built for a WASM target.
pub mod wasm;

/// The host trait defines methods a Stylus contract can use to interact
/// with a host environment, such as the EVM. It is a composition
/// of traits with different access to host values and modifications.
/// Stylus contracts in the SDK parametrized by a host trait allow for safe access
/// to hostios without the need for global invocations. The host trait may be implemented
/// by test frameworks as an easier way of mocking hostio invocations for testing
/// Stylus contracts.
pub trait Host:
    CryptographyAccess
    + CalldataAccess
    + DeploymentAccess
    + StorageAccess
    + CallAccess
    + BlockAccess
    + ChainAccess
    + AccountAccess
    + MemoryAccess
    + MessageAccess
    + MeteringAccess
{
}

/// Defines a trait that allows a Stylus contract to access its host safely.
pub trait HostAccess {
    /// The associated host type for a Stylus contract.
    type Host: Host;
    /// Provides access to the parametrized host of a contract, giving access
    /// to all the desired hostios from the user.
    fn vm(&self) -> &Self::Host;
}

/// Provides access to native cryptography extensions provided by
/// a Stylus contract host, such as keccak256.
pub trait CryptographyAccess {
    /// Efficiently computes the [`keccak256`] hash of the given preimage.
    /// The semantics are equivalent to that of the EVM's [`SHA3`] opcode.
    ///
    /// [`keccak256`]: https://en.wikipedia.org/wiki/SHA-3
    /// [`SHA3`]: https://www.evm.codes/#20
    fn native_keccak256(&self, input: &[u8]) -> B256;
}

/// Provides access to host methods relating to the accessing the calldata
/// of a Stylus contract transaction.
pub trait CalldataAccess {
    /// Reads the program calldata. The semantics are equivalent to that of the EVM's
    /// [`CALLDATA_COPY`] opcode when requesting the entirety of the current call's calldata.
    ///
    /// [`CALLDATA_COPY`]: https://www.evm.codes/#37
    fn read_args(&self, len: usize) -> Vec<u8>;
    /// Copies the bytes of the last EVM call or deployment return result. Does not revert if out of
    /// bounds, but rather copies the overlapping portion. The semantics are otherwise equivalent
    /// to that of the EVM's [`RETURN_DATA_COPY`] opcode.
    ///
    /// Returns the number of bytes written.
    ///
    /// [`RETURN_DATA_COPY`]: https://www.evm.codes/#3e
    fn read_return_data(&self, offset: usize, size: Option<usize>) -> Vec<u8>;
    /// Returns the length of the last EVM call or deployment return result, or `0` if neither have
    /// happened during the program's execution. The semantics are equivalent to that of the EVM's
    /// [`RETURN_DATA_SIZE`] opcode.
    ///
    /// [`RETURN_DATA_SIZE`]: https://www.evm.codes/#3d
    fn return_data_size(&self) -> usize;
    /// Writes the final return data. If not called before the program exists, the return data will
    /// be 0 bytes long. Note that this hostio does not cause the program to exit, which happens
    /// naturally when `user_entrypoint` returns.
    fn write_result(&self, data: &[u8]);
}

/// Provides access to programmatic creation of contracts via the host environment's CREATE
/// and CREATE2 opcodes in the EVM.
///
/// # Safety
/// These methods should only be used in advanced cases when lowest-level access
/// to create1 and create2 opcodes is needed. Using the methods by themselves will not protect
/// against reentrancy safety, storage aliasing, or cache flushing. For safe contract deployment,
/// utilize a [`RawDeploy`] struct instead.
pub unsafe trait DeploymentAccess {
    /// Deploys a new contract using the init code provided, which the EVM executes to construct
    /// the code of the newly deployed contract. The init code must be written in EVM bytecode, but
    /// the code it deploys can be that of a Stylus program. The code returned will be treated as
    /// WASM if it begins with the EOF-inspired header `0xEFF000`. Otherwise the code will be
    /// interpreted as that of a traditional EVM-style contract. See [`Deploying Stylus Programs`]
    /// for more information on writing init code.
    ///
    /// On success, this hostio returns the address of the newly created account whose address is
    /// a function of the sender and nonce. On failure the address will be `0`, `return_data_len`
    /// will store the length of the revert data, the bytes of which can be read via the
    /// `read_return_data` hostio. The semantics are equivalent to that of the EVM's [`CREATE`]
    /// opcode, which notably includes the exact address returned.
    ///
    /// [`Deploying Stylus Programs`]: https://docs.arbitrum.io/stylus/quickstart
    /// [`CREATE`]: https://www.evm.codes/#f0
    ///
    /// # Safety
    /// This method should only be used in advanced cases when lowest-level access to create1 is required.
    /// Safe usage needs to consider reentrancy, storage aliasing, and cache flushing.
    /// utilize a [`RawDeploy`] struct instead for safety.
    unsafe fn create1(
        &self,
        code: Address,
        endowment: U256,
        contract: &mut Address,
        revert_data_len: &mut usize,
    );
    /// Deploys a new contract using the init code provided, which the EVM executes to construct
    /// the code of the newly deployed contract. The init code must be written in EVM bytecode, but
    /// the code it deploys can be that of a Stylus program. The code returned will be treated as
    /// WASM if it begins with the EOF-inspired header `0xEFF000`. Otherwise the code will be
    /// interpreted as that of a traditional EVM-style contract. See [`Deploying Stylus Programs`]
    /// for more information on writing init code.
    ///
    /// On success, this hostio returns the address of the newly created account whose address is a
    /// function of the sender, salt, and init code. On failure the address will be `0`,
    /// `return_data_len` will store the length of the revert data, the bytes of which can be read
    /// via the `read_return_data` hostio. The semantics are equivalent to that of the EVM's
    /// `[CREATE2`] opcode, which notably includes the exact address returned.
    ///
    /// [`Deploying Stylus Programs`]: https://docs.arbitrum.io/stylus/quickstart
    /// [`CREATE2`]: https://www.evm.codes/#f5
    ///
    /// # Safety
    /// This method should only be used in advanced cases when lowest-level access to create2 is required.
    /// Safe usage needs to consider reentrancy, storage aliasing, and cache flushing.
    /// utilize a [`RawDeploy`] struct instead for safety.
    unsafe fn create2(
        &self,
        code: Address,
        endowment: U256,
        salt: B256,
        contract: &mut Address,
        revert_data_len: &mut usize,
    );
}

/// Provides access to storage access and mutation via host methods.
pub trait StorageAccess {
    /// Emits an EVM log with the given number of topics and data, the first bytes of which should
    /// be the 32-byte-aligned topic data. The semantics are equivalent to that of the EVM's
    /// [`LOG0`], [`LOG1`], [`LOG2`], [`LOG3`], and [`LOG4`] opcodes based on the number of topics
    /// specified. Requesting more than `4` topics will induce a revert.
    ///
    /// [`LOG0`]: https://www.evm.codes/#a0
    /// [`LOG1`]: https://www.evm.codes/#a1
    /// [`LOG2`]: https://www.evm.codes/#a2
    /// [`LOG3`]: https://www.evm.codes/#a3
    /// [`LOG4`]: https://www.evm.codes/#a4
    fn emit_log(&self, input: &[u8], num_topics: usize);
    /// Reads a 32-byte value from permanent storage. Stylus's storage format is identical to
    /// that of the EVM. This means that, under the hood, this hostio is accessing the 32-byte
    /// value stored in the EVM state trie at offset `key`, which will be `0` when not previously
    /// set. The semantics, then, are equivalent to that of the EVM's [`SLOAD`] opcode.
    ///
    /// Note: the Stylus VM implements storage caching. This means that repeated calls to the same key
    /// will cost less than in the EVM.
    ///
    /// [`SLOAD`]: https://www.evm.codes/#54
    fn storage_load_bytes32(&self, key: U256) -> B256;
    /// Writes a 32-byte value to the permanent storage cache. Stylus's storage format is identical to that
    /// of the EVM. This means that, under the hood, this hostio represents storing a 32-byte value into
    /// the EVM state trie at offset `key`. Refunds are tabulated exactly as in the EVM. The semantics, then,
    /// are equivalent to that of the EVM's [`SSTORE`] opcode.
    ///
    /// Note: because the value is cached, one must call `storage_flush_cache` to persist it.
    ///
    /// [`SSTORE`]: https://www.evm.codes/#55
    ///
    /// # Safety
    /// May alias storage.
    unsafe fn storage_cache_bytes32(&self, key: U256, value: B256);
    /// Persists any dirty values in the storage cache to the EVM state trie, dropping the cache entirely if requested.
    /// Analogous to repeated invocations of [`SSTORE`].
    ///
    /// [`SSTORE`]: https://www.evm.codes/#55
    fn flush_cache(&self, clear: bool);
}

/// Provides access to calling other contracts using host semantics.
///
/// # Safety
/// These methods should only be used in advanced cases when lowest-level access
/// to call, static_call, and delegate_call methods is required. Using the methods by themselves will not protect
/// against reentrancy safety, storage aliasing, or cache flushing. For safe contract calls,
/// utilize a [`RawCall`] struct instead.
pub unsafe trait CallAccess {
    /// Calls the contract at the given address with options for passing value and to limit the
    /// amount of gas supplied. The return status indicates whether the call succeeded, and is
    /// nonzero on failure.
    ///
    /// In both cases `return_data_len` will store the length of the result, the bytes of which can
    /// be read via the `read_return_data` hostio. The bytes are not returned directly so that the
    /// programmer can potentially save gas by choosing which subset of the return result they'd
    /// like to copy.
    ///
    /// The semantics are equivalent to that of the EVM's [`CALL`] opcode, including callvalue
    /// stipends and the 63/64 gas rule. This means that supplying the `u64::MAX` gas can be used
    /// to send as much as possible.
    ///
    /// [`CALL`]: https://www.evm.codes/#f1
    ///
    /// # Safety
    /// This method should only be used in advanced cases when lowest-level access to calls is required.
    /// Safe usage needs to consider reentrancy, storage aliasing, and cache flushing.
    /// utilize a [`RawCall`] struct instead for safety.
    unsafe fn call_contract(
        &self,
        to: Address,
        data: &[u8],
        value: U256,
        gas: u64,
        outs_len: &mut usize,
    ) -> u8;
    /// Static calls the contract at the given address, with the option to limit the amount of gas
    /// supplied. The return status indicates whether the call succeeded, and is nonzero on
    /// failure.
    ///
    /// In both cases `return_data_len` will store the length of the result, the bytes of which can
    /// be read via the `read_return_data` hostio. The bytes are not returned directly so that the
    /// programmer can potentially save gas by choosing which subset of the return result they'd
    /// like to copy.
    ///
    /// The semantics are equivalent to that of the EVM's [`STATIC_CALL`] opcode, including the
    /// 63/64 gas rule. This means that supplying `u64::MAX` gas can be used to send as much as
    /// possible.
    ///
    /// [`STATIC_CALL`]: https://www.evm.codes/#FA
    ///
    /// # Safety
    /// This method should only be used in advanced cases when lowest-level access to calls is required.
    /// Safe usage needs to consider reentrancy, storage aliasing, and cache flushing.
    /// utilize a [`RawCall`] struct instead for safety.
    unsafe fn static_call_contract(
        &self,
        to: Address,
        data: &[u8],
        gas: u64,
        outs_len: &mut usize,
    ) -> u8;
    /// Delegate calls the contract at the given address, with the option to limit the amount of
    /// gas supplied. The return status indicates whether the call succeeded, and is nonzero on
    /// failure.
    ///
    /// In both cases `return_data_len` will store the length of the result, the bytes of which
    /// can be read via the `read_return_data` hostio. The bytes are not returned directly so that
    /// the programmer can potentially save gas by choosing which subset of the return result
    /// they'd like to copy.
    ///
    /// The semantics are equivalent to that of the EVM's [`DELEGATE_CALL`] opcode, including the
    /// 63/64 gas rule. This means that supplying `u64::MAX` gas can be used to send as much as
    /// possible.
    ///
    /// [`DELEGATE_CALL`]: https://www.evm.codes/#F4
    ///
    /// # Safety
    /// This method should only be used in advanced cases when lowest-level access to calls is required.
    /// Safe usage needs to consider reentrancy, storage aliasing, and cache flushing.
    /// utilize a [`RawCall`] struct instead for safety.
    unsafe fn delegate_call_contract(
        &self,
        to: Address,
        data: &[u8],
        gas: u64,
        outs_len: &mut usize,
    ) -> u8;
}

/// Provides access to host methods relating to the block a transactions
/// to a Stylus contract is included in.
pub trait BlockAccess {
    /// Gets the basefee of the current block. The semantics are equivalent to that of the EVM's
    /// [`BASEFEE`] opcode.
    ///
    /// [`BASEFEE`]: https://www.evm.codes/#48
    fn block_basefee(&self) -> U256;
    /// Gets the coinbase of the current block, which on Arbitrum chains is the L1 batch poster's
    /// address. This differs from Ethereum where the validator including the transaction
    /// determines the coinbase.
    fn block_coinbase(&self) -> Address;
    /// Gets a bounded estimate of the L1 block number at which the Sequencer sequenced the
    /// transaction. See [`Block Numbers and Time`] for more information on how this value is
    /// determined.
    ///
    /// [`Block Numbers and Time`]: https://developer.arbitrum.io/time
    fn block_number(&self) -> u64;
    /// Gets a bounded estimate of the Unix timestamp at which the Sequencer sequenced the
    /// transaction. See [`Block Numbers and Time`] for more information on how this value is
    /// determined.
    ///
    /// [`Block Numbers and Time`]: https://developer.arbitrum.io/time
    fn block_timestamp(&self) -> u64;
    /// Gets the gas limit of the current block. The semantics are equivalent to that of the EVM's
    /// [`GAS_LIMIT`] opcode. Note that as of the time of this writing, `evm.codes` incorrectly
    /// implies that the opcode returns the gas limit of the current transaction.  When in doubt,
    /// consult [`The Ethereum Yellow Paper`].
    ///
    /// [`GAS_LIMIT`]: https://www.evm.codes/#45
    /// [`The Ethereum Yellow Paper`]: https://ethereum.github.io/yellowpaper/paper.pdf
    fn block_gas_limit(&self) -> u64;
}

/// Provides access to the chain details of the host environment.
pub trait ChainAccess {
    /// Gets the unique chain identifier of the Arbitrum chain. The semantics are equivalent to
    /// that of the EVM's [`CHAIN_ID`] opcode.
    ///
    /// [`CHAIN_ID`]: https://www.evm.codes/#46
    fn chain_id(&self) -> u64;
}

/// Provides access to account details of addresses of the host environment.
pub trait AccountAccess {
    /// Gets the ETH balance in wei of the account at the given address.
    /// The semantics are equivalent to that of the EVM's [`BALANCE`] opcode.
    ///
    /// [`BALANCE`]: https://www.evm.codes/#31
    fn balance(&self, account: Address) -> U256;
    /// Gets the address of the current program. The semantics are equivalent to that of the EVM's
    /// [`ADDRESS`] opcode.
    ///
    /// [`ADDRESS`]: https://www.evm.codes/#30
    fn contract_address(&self) -> Address;
    /// Gets a subset of the code from the account at the given address. The semantics are identical to that
    /// of the EVM's [`EXT_CODE_COPY`] opcode, aside from one small detail: the write to the buffer `dest` will
    /// stop after the last byte is written. This is unlike the EVM, which right pads with zeros in this scenario.
    /// The return value is the number of bytes written, which allows the caller to detect if this has occurred.
    ///
    /// [`EXT_CODE_COPY`]: https://www.evm.codes/#3C
    fn code(&self, account: Address) -> Vec<u8>;
    /// Gets the size of the code in bytes at the given address. The semantics are equivalent
    /// to that of the EVM's [`EXT_CODESIZE`].
    ///
    /// [`EXT_CODESIZE`]: https://www.evm.codes/#3B
    fn code_size(&self, account: Address) -> usize;
    /// Gets the code hash of the account at the given address. The semantics are equivalent
    /// to that of the EVM's [`EXT_CODEHASH`] opcode. Note that the code hash of an account without
    /// code will be the empty hash
    /// `keccak("") = c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470`.
    ///
    /// [`EXT_CODEHASH`]: https://www.evm.codes/#3F
    fn codehash(&self, account: Address) -> B256;
}

/// Provides the ability to pay for memory growth of a Stylus contract.
pub trait MemoryAccess {
    /// The `entrypoint!` macro handles importing this hostio, which is required if the
    /// program's memory grows. Otherwise compilation through the `ArbWasm` precompile will revert.
    /// Internally the Stylus VM forces calls to this hostio whenever new WASM pages are allocated.
    /// Calls made voluntarily will unproductively consume gas.
    fn pay_for_memory_grow(&self, pages: u16);
}

/// Provides access to transaction details of a Stylus contract.
pub trait MessageAccess {
    /// Gets the address of the account that called the program. For normal L2-to-L2 transactions
    /// the semantics are equivalent to that of the EVM's [`CALLER`] opcode, including in cases
    /// arising from [`DELEGATE_CALL`].
    ///
    /// For L1-to-L2 retryable ticket transactions, the top-level sender's address will be aliased.
    /// See [`Retryable Ticket Address Aliasing`] for more information on how this works.
    ///
    /// [`CALLER`]: https://www.evm.codes/#33
    /// [`DELEGATE_CALL`]: https://www.evm.codes/#f4
    /// [`Retryable Ticket Address Aliasing`]: https://developer.arbitrum.io/arbos/l1-to-l2-messaging#address-aliasing
    fn msg_sender(&self) -> Address;
    /// Whether the current call is reentrant.
    fn msg_reentrant(&self) -> bool;
    /// Get the ETH value in wei sent to the program. The semantics are equivalent to that of the
    /// EVM's [`CALLVALUE`] opcode.
    ///
    /// [`CALLVALUE`]: https://www.evm.codes/#34
    fn msg_value(&self) -> U256;
    /// Gets the top-level sender of the transaction. The semantics are equivalent to that of the
    /// EVM's [`ORIGIN`] opcode.
    ///
    /// [`ORIGIN`]: https://www.evm.codes/#32
    fn tx_origin(&self) -> Address;
}

/// Provides access to metering values such as EVM gas and Stylus ink used and remaining,
/// as well as details of their prices based on the host environment.
pub trait MeteringAccess {
    /// Gets the amount of gas left after paying for the cost of this hostio. The semantics are
    /// equivalent to that of the EVM's [`GAS`] opcode.
    ///
    /// [`GAS`]: https://www.evm.codes/#5a
    fn evm_gas_left(&self) -> u64;
    /// Gets the amount of ink remaining after paying for the cost of this hostio. The semantics
    /// are equivalent to that of the EVM's [`GAS`] opcode, except the units are in ink. See
    /// [`Ink and Gas`] for more information on Stylus's compute pricing.
    ///
    /// [`GAS`]: https://www.evm.codes/#5a
    /// [`Ink and Gas`]: https://docs.arbitrum.io/stylus/concepts/gas-metering
    fn evm_ink_left(&self) -> u64;
    /// Gets the gas price in wei per gas, which on Arbitrum chains equals the basefee. The
    /// semantics are equivalent to that of the EVM's [`GAS_PRICE`] opcode.
    ///
    /// [`GAS_PRICE`]: https://www.evm.codes/#3A
    fn tx_gas_price(&self) -> U256;
    /// Gets the price of ink in evm gas basis points. See [`Ink and Gas`] for more information on
    /// Stylus's compute-pricing model.
    ///
    /// [`Ink and Gas`]: https://docs.arbitrum.io/stylus/concepts/gas-metering
    fn tx_ink_price(&self) -> u32;
}
