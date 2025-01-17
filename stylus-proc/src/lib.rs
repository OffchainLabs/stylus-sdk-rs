// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Procedural macros for [The Stylus SDK][sdk].
//!
//! You can import these via
//!
//! ```
//! use stylus_sdk::prelude::*;
//! ```
//!
//! For a guided exploration of the features, please see the comprehensive [Feature Overview][overview].
//!
//! [overview]: https://docs.arbitrum.io/stylus/reference/rust-sdk-guide#calls
//! [sdk]: https://docs.rs/stylus-sdk/latest/stylus_sdk/index.html

#![warn(missing_docs)]

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

/// Generates a pretty error message.
/// Note that this macro is declared before all modules so that they can use it.
macro_rules! error {
    ($tokens:expr, $($msg:expr),+ $(,)?) => {{
        let error = syn::Error::new(syn::spanned::Spanned::span(&$tokens), format!($($msg),+));
        return error.to_compile_error().into();
    }};
    (@ $tokens:expr, $($msg:expr),+ $(,)?) => {{
        return Err(syn::Error::new(syn::spanned::Spanned::span(&$tokens), format!($($msg),+)))
    }};
}

mod consts;
mod impls;
mod imports;
mod macros;
mod types;
mod utils;

/// Allows a Rust `struct` to be used in persistent storage.
///
/// ```
/// extern crate alloc;
/// # use stylus_sdk::storage::{StorageAddress, StorageBool};
/// # use stylus_proc::storage;
/// # use stylus_sdk::prelude::*;
/// #[storage]
/// pub struct Contract {
///    owner: StorageAddress,
///    active: StorageBool,
///    sub_struct: SubStruct,
///}
///
///#[storage]
///pub struct SubStruct {
///    number: StorageBool,
///}
/// ```
///
/// Each field must implement [`StorageType`]. This includes other structs, which will
/// implement the `trait` automatically when [`#[storage]`][storage] is applied.
///
/// One may even implement [`StorageType`] to define custom storage entries, though this is rarely necessary
/// since the [Stylus SDK][sdk] intends to include all standard Solidity types out-of-the-box.
///
/// Please refer to the [SDK Feature Overview][overview] for more information on defining storage.
///
/// [storage]: macro@storage
/// [`StorageType`]: https://docs.rs/stylus-sdk/latest/stylus_sdk/storage/trait.StorageType.html
/// [overview]: https://docs.arbitrum.io/stylus/reference/rust-sdk-guide#storage
/// [sdk]: https://docs.rs/stylus-sdk/latest/stylus_sdk/index.html
#[proc_macro_attribute]
#[proc_macro_error]
pub fn storage(attr: TokenStream, input: TokenStream) -> TokenStream {
    macros::storage(attr, input)
}

#[doc(hidden)]
#[deprecated = "please use `#[storage]` instead"]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn solidity_storage(attr: TokenStream, input: TokenStream) -> TokenStream {
    macros::storage(attr, input)
}

/// The types in [`#[storage]`][storage] are laid out in the EVM state trie exactly
/// as they are in [Solidity][solidity]. This means that the fields of a `struct` definition will map
/// to the same storage slots as they would in EVM programming languages. Hence, it is often nice to
/// define types using Solidity syntax, which makes this guarantee easier to see.
///
/// ```
/// extern crate alloc;
/// # use stylus_sdk::prelude::*;
/// # use stylus_proc::sol_storage;
/// sol_storage! {
///     pub struct Contract {
///         address owner;                      // becomes a StorageAddress
///         bool active;                        // becomes a StorageBool
///         SubStruct sub_struct;
///     }
///
///     pub struct SubStruct {
///         // other solidity fields, such as
///         mapping(address => uint) balances;  // becomes a StorageMap
///         Delegate[] delegates;               // becomes a StorageVec
///     }
///     pub struct Delegate {
///     }
/// }
/// ```
///
/// The above will expand to equivalent definitions in Rust, with each structure implementing the [`StorageType`]
/// `trait`. Many contracts, like [the ERC 20 example][erc20], do exactly this.
///
/// Because the layout is identical to [Solidity's][solidity], existing Solidity smart contracts can
/// upgrade to Rust without fear of storage slots not lining up. You simply copy-paste your type definitions.
///
/// Note that one exception to this storage layout guarantee is contracts which utilize
/// inheritance. The current solution in Stylus using `#[borrow]` and `#[inherits(...)]` packs
/// nested (inherited) structs into their own slots. This is consistent with regular struct nesting
/// in solidity, but not inherited structs. We plan to revisit this behavior in an upcoming
/// release.
///
/// Consequently, the order of fields will affect the JSON ABIs produced that explorers and tooling might use.
/// Most developers don't need to worry about this though and can freely order their types when working on a
/// Rust contract from scratch.
///
///
/// Please refer to the [SDK Feature Overview][overview] for more information on defining storage.
///
/// [storage]: macro@storage
/// [`StorageType`]: https://docs.rs/stylus-sdk/latest/stylus_sdk/storage/trait.StorageType.html
/// [solidity]: https://docs.soliditylang.org/en/latest/internals/layout_in_storage.html
/// [overview]: https://docs.arbitrum.io/stylus/reference/rust-sdk-guide#erase-and-deriveerase
/// [erc20]: https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/examples/erc20/src/main.rs
#[proc_macro]
#[proc_macro_error]
pub fn sol_storage(input: TokenStream) -> TokenStream {
    macros::sol_storage(input)
}

/// Facilitates calls to other contracts.
///
/// This macro defines a `struct` for each of the Solidity interfaces provided.
///
/// ```
/// # use stylus_proc::sol_interface;
/// sol_interface! {
///     interface IService {
///         function makePayment(address user) external payable returns (string);
///         function getConstant() external pure returns (bytes32);
///     }
///
///     interface ITree {
///         // other interface methods
///     }
/// }
/// ```
///
/// The above will define `IService` and `ITree` for calling the methods of the two contracts.
///
/// For example, `IService` will have a `make_payment` method that accepts an [`Address`] and returns a [`B256`].
///
/// Currently only functions are supported, and any other items in the interface will cause an
/// error. Additionally, each function must be marked `external`. Inheritance is not supported.
///
/// ```
/// use stylus_sdk::call::{Call, Error};
/// use alloy_primitives::Address;
/// # use stylus_proc::sol_interface;
///
/// # sol_interface! {
/// #     interface IService {
/// #         function makePayment(address user) external payable returns (string);
/// #     }
/// # }
/// # mod evm { pub fn gas_left() -> u64 { 100 } }
/// # mod msg { pub fn value() -> alloy_primitives::U256 { 100.try_into().unwrap() } }
/// pub fn do_call(account: IService, user: Address) -> Result<String, Error> {
///     let config = Call::new()
///         .gas(evm::gas_left() / 2)       // limit to half the gas left
///         .value(msg::value());           // set the callvalue
///
///     account.make_payment(config, user)  // note the snake case
/// }
/// ```
///
/// Observe the casing change. [`sol_interface!`] computes the selector based on the exact name passed in,
/// which should almost always be `camelCase`. For aesthetics, the rust functions will instead use `snake_case`.
///
/// Note that structs may be used, as return types for example. Trying to reference structs using
/// the Solidity path separator (`module.MyStruct`) is supported and paths will be converted to
/// Rust syntax (`module::MyStruct`).
///
/// # Reentrant calls
///
/// Contracts that opt into reentrancy via the `reentrant` feature flag require extra care.
/// When enabled, cross-contract calls must [`flush`] or [`clear`] the [`StorageCache`] to safeguard state.
/// This happens automatically via the type system.
///
/// ```
/// # extern crate alloc;
/// # use stylus_sdk::call::Call;
/// # use stylus_sdk::prelude::*;
/// # use stylus_proc::{entrypoint, public, sol_interface, storage};
/// sol_interface! {
///     interface IMethods {
///         function pureFoo() external pure;
///         function viewFoo() external view;
///         function writeFoo() external;
///         function payableFoo() external payable;
///     }
/// }
///
/// #[entrypoint] #[storage] struct Contract {}
/// #[public]
/// impl Contract {
///     pub fn call_pure(&self, methods: IMethods) -> Result<(), Vec<u8>> {
///         Ok(methods.pure_foo(self)?)    // `pure` methods might lie about not being `view`
///     }
///
///     pub fn call_view(&self, methods: IMethods) -> Result<(), Vec<u8>> {
///         Ok(methods.view_foo(self)?)
///     }
///
///     pub fn call_write(&mut self, methods: IMethods) -> Result<(), Vec<u8>> {
///         methods.view_foo(&mut *self)?;       // allows `pure` and `view` methods too
///         Ok(methods.write_foo(self)?)
///     }
///
///     #[payable]
///     pub fn call_payable(&mut self, methods: IMethods) -> Result<(), Vec<u8>> {
///         methods.write_foo(Call::new_in(self))?;   // these are the same
///         Ok(methods.payable_foo(self)?)            // ------------------
///     }
/// }
/// ```
///
/// In the above, we're able to pass `&self` and `&mut self` because `Contract` implements
/// [`TopLevelStorage`], which means that a reference to it entails access to the entirety of
/// the contract's state. This is the reason it is sound to make a call, since it ensures all
/// cached values are invalidated and/or persisted to state at the right time.
///
/// When writing Stylus libraries, a type might not be [`TopLevelStorage`] and therefore
/// `&self` or `&mut self` won't work. Building a [`Call`] from a generic parameter is the usual solution.
///
/// ```
/// use stylus_sdk::{call::{Call, Error}, storage::TopLevelStorage};
/// use alloy_primitives::Address;
/// # use stylus_proc::sol_interface;
///
/// # sol_interface! {
/// #     interface IService {
/// #         function makePayment(address user) external payable returns (string);
/// #     }
/// # }
/// # mod evm { pub fn gas_left() -> u64 { 100 } }
/// # mod msg { pub fn value() -> alloy_primitives::U256 { 100.try_into().unwrap() } }
/// pub fn do_call(
///     storage: &mut impl TopLevelStorage,  // can be generic, but often just &mut self
///     account: IService,                   // serializes as an Address
///     user: Address,
/// ) -> Result<String, Error> {
///
///     let config = Call::new_in(storage)
///         .gas(evm::gas_left() / 2)        // limit to half the gas left
///         .value(msg::value());            // set the callvalue
///
///     account.make_payment(config, user)   // note the snake case
/// }
/// ```
///
/// Note that in the context of a [`#[public]`][public] call, the `&mut impl` argument will correctly
/// distinguish the method as being `write` or `payable`. This means you can write library code that will
/// work regardless of whether the `reentrant` feature flag is enabled.
///
/// [sol_interface]: macro@sol_interface
/// [public]: macro@public
/// [`TopLevelStorage`]: https://docs.rs/stylus-sdk/latest/stylus_sdk/storage/trait.TopLevelStorage.html
/// [`StorageCache`]: https://docs.rs/stylus-sdk/latest/stylus_sdk/storage/struct.StorageCache.html
/// [`flush`]: https://docs.rs/stylus-sdk/latest/stylus_sdk/storage/struct.StorageCache.html#method.flush
/// [`clear`]: https://docs.rs/stylus-sdk/latest/stylus_sdk/storage/struct.StorageCache.html#method.clear
/// [`Address`]: https://docs.rs/alloy-primitives/latest/alloy_primitives/struct.Address.html
/// [`B256`]: https://docs.rs/alloy-primitives/latest/alloy_primitives/aliases/type.B256.html
/// [`Call`]: https://docs.rs/stylus-sdk/latest/stylus_sdk/call/struct.Call.html
#[proc_macro]
#[proc_macro_error]
pub fn sol_interface(input: TokenStream) -> TokenStream {
    macros::sol_interface(input)
}

/// Some [`StorageType`] values implement [`Erase`], which provides an [`erase()`] method for clearing state.
/// [The Stylus SDK][sdk] implements [`Erase`] for all primitives, and for vectors of primitives, but not for maps.
/// This is because a Solidity mapping does not provide iteration, and so it's generally impossible to
/// know which slots to clear.
///
/// Structs may also be [`Erase`] if all of the fields are. `#[derive(Erase)]`
/// lets you do this automatically.
///
/// ```
/// extern crate alloc;
/// # use stylus_proc::{Erase, sol_storage};
/// sol_storage! {
///    #[derive(Erase)]
///    pub struct Contract {
///        address owner;              // can erase primitive
///        uint256[] hashes;           // can erase vector of primitive
///    }
///
///    pub struct NotErase {
///        mapping(address => uint) balances; // can't erase a map
///        mapping(uint => uint)[] roots;     // can't erase vector of maps
///    }
/// }
/// ```
///
/// You can also implement [`Erase`] manually if desired. Note that the reason we care about [`Erase`]
/// at all is that you get storage refunds when clearing state, lowering fees. There's also
/// minor implications for storage patterns using `unsafe` Rust.
///
/// Please refer to the [SDK Feature Overview][overview] for more information on defining storage.
///
/// [`StorageType`]: https://docs.rs/stylus-sdk/latest/stylus_sdk/storage/trait.StorageType.html
/// [`Erase`]: https://docs.rs/stylus-sdk/latest/stylus_sdk/storage/trait.Erase.html
/// [`erase()`]: https://docs.rs/stylus-sdk/latest/stylus_sdk/storage/trait.Erase.html#tymethod.erase
/// [overview]: https://docs.arbitrum.io/stylus/reference/rust-sdk-guide#storage
/// [sdk]: https://docs.rs/stylus-sdk/latest/stylus_sdk/index.html
#[proc_macro_derive(Erase)]
#[proc_macro_error]
pub fn derive_erase(input: TokenStream) -> TokenStream {
    macros::derive_erase(input)
}

/// Allows an error `enum` to be used in method signatures.
///
/// ```
/// # use alloy_sol_types::sol;
/// # use stylus_proc::{public, SolidityError};
/// # extern crate alloc;
/// sol! {
///     error InsufficientBalance(address from, uint256 have, uint256 want);
///     error InsufficientAllowance(address owner, address spender, uint256 have, uint256 want);
/// }
///
/// #[derive(SolidityError)]
/// pub enum Erc20Error {
///     InsufficientBalance(InsufficientBalance),
///     InsufficientAllowance(InsufficientAllowance),
/// }
///
/// # struct Contract {}
/// #[public]
/// impl Contract {
///     pub fn fallible_method() -> Result<(), Erc20Error> {
///         // code that might revert
/// #       Ok(())
///     }
/// }
/// ```
///
/// Under the hood, the above macro works by implementing `From<Erc20Error>` for `Vec<u8>`
/// along with printing code for abi-export.
#[proc_macro_derive(SolidityError)]
#[proc_macro_error]
pub fn derive_solidity_error(input: TokenStream) -> TokenStream {
    macros::derive_solidity_error(input)
}

/// Defines the entrypoint, which is where Stylus execution begins.
/// Without it the contract will fail to pass [`cargo stylus check`][check].
/// Most commonly this macro is used to annotate the top level storage `struct`.
///
/// ```
/// # extern crate alloc;
/// # use stylus_proc::{entrypoint, public, sol_storage};
/// # use stylus_sdk::prelude::*;
/// sol_storage! {
///     #[entrypoint]
///     pub struct Contract {
///     }
///
///     // only one entrypoint is allowed
///     pub struct SubStruct {
///     }
/// }
/// # #[public] impl Contract {}
/// ```
///
/// The above will make the public methods of Contract the first to consider during invocation.
/// See [`#[public]`][public] for more information on method selection.
///
/// # Bytes-in, bytes-out programming
///
/// A less common usage of [`#[entrypoint]`][entrypoint] is for low-level, bytes-in bytes-out programming.
/// When applied to a free-standing function, a different way of writing smart contracts becomes possible,
/// wherein the Stylus SDK's macros and storage types are entirely optional.
///
/// ```
/// extern crate alloc;
/// # use stylus_sdk::ArbResult;
/// # use stylus_proc::entrypoint;
/// # use stylus_sdk::prelude::*;
/// #[entrypoint]
/// fn entrypoint(calldata: Vec<u8>, _: alloc::boxed::Box<dyn stylus_sdk::host::Host>) -> ArbResult {
///     // bytes-in, bytes-out programming
/// #   Ok(Vec::new())
/// }
/// ```
///
/// # Reentrancy
///
/// If a contract calls another that then calls the first, it is said to be reentrant. By default,
/// all Stylus programs revert when this happens. However, you can opt out of this behavior by
/// recompiling with the `reentrant` flag.
///
/// ```toml
/// stylus_sdk = { version = "0.3.0", features = ["reentrant"] }
/// ```
///
/// This is dangerous, and should be done only after careful review -- ideally by 3rd-party auditors.
/// Numerous exploits and hacks have in Web3 are attributable to developers misusing or not fully
/// understanding reentrant patterns.
///
/// If enabled, the Stylus SDK will flush the storage cache in between reentrant calls, persisting values
/// to state that might be used by inner calls. Note that preventing storage invalidation is only part
/// of the battle in the fight against exploits. You can tell if a call is reentrant via
/// [`msg::reentrant`][reentrant], and condition your business logic accordingly.
///
/// # [`TopLevelStorage`]
///
/// The [`#[entrypoint]`][entrypoint] macro will automatically implement the [`TopLevelStorage`] `trait`
/// for the annotated `struct`. The single type implementing [`TopLevelStorage`] is special in that
/// mutable access to it represents mutable access to the entire program's state.
/// This has implications for calls via [`sol_interface`].
///
/// [`TopLevelStorage`]: https://docs.rs/stylus-sdk/latest/stylus_sdk/storage/trait.TopLevelStorage.html
/// [`sol_interface`]: macro@sol_interface
/// [entrypoint]: macro@entrypoint
/// [reentrant]: https://docs.rs/stylus-sdk/latest/stylus_sdk/msg/fn.reentrant.html
/// [public]: macro@public
/// [check]: https://github.com/OffchainLabs/cargo-stylus#developing-with-stylus
#[proc_macro_attribute]
#[proc_macro_error]
pub fn entrypoint(attr: TokenStream, input: TokenStream) -> TokenStream {
    macros::entrypoint(attr, input)
}

/// Just as with storage, Stylus SDK methods are Solidity ABI-equivalent. This means that contracts written
/// in different programming languages are fully interoperable. You can even automatically export your
/// Rust contract as a Solidity interface so that others can add it to their Solidity projects.
///
/// This macro makes methods "public" so that other contracts can call them by implementing the [`Router`] trait.
///
/// ```
/// # extern crate alloc;
/// # use stylus_sdk::storage::StorageAddress;
/// # use stylus_proc::public;
/// # use alloy_primitives::Address;
/// # struct Contract {
/// #     owner: StorageAddress,
/// # }
/// #[public]
/// impl Contract {
///     // our owner method is now callable by other contracts
///     pub fn owner(&self) -> Result<Address, Vec<u8>> {
///         Ok(self.owner.get())
///     }
/// }
///
/// impl Contract {
///     // our set_owner method is not
///     pub fn set_owner(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
///         // ...
/// #       Ok(())
///     }
/// }
/// ```
///
/// In is example, [`Vec<u8>`] becomes the program's revert data.
///
/// # [`#[payable]`][payable]
///
/// As in Solidity, methods may accept ETH as call value.
///
/// ```
/// # extern crate alloc;
/// # use alloy_primitives::Address;
/// # use stylus_proc::{entrypoint, public, storage};
/// # use stylus_sdk::prelude::*;
/// # #[entrypoint] #[storage] struct Contract { #[borrow] erc20: Erc20, }
/// # mod msg {
/// #     use alloy_primitives::Address;
/// #     pub fn sender() -> Address { Address::ZERO }
/// #     pub fn value() -> u32 { 0 }
/// # }
/// #[public]
/// impl Contract {
///     #[payable]
///     pub fn credit(&mut self) -> Result<(), Vec<u8>> {
///         self.erc20.add_balance(msg::sender(), msg::value())
///     }
/// }
/// # #[storage] struct Erc20;
/// # #[public]
/// # impl Erc20 {
/// #     pub fn add_balance(&self, sender: Address, value: u32) -> Result<(), Vec<u8>> {
/// #         Ok(())
/// #     }
/// # }
/// ```
///
/// In the above, [msg::value][value] is the amount of ETH passed to the contract in wei, which may be used
/// to pay for something depending on the contract's business logic. Note that you have to annotate the method
/// with [`#[payable]`][payable], or else calls to it will revert. This is required as a safety measure
/// to prevent users losing funds to methods that didn't intend to accept ether.
///
/// # [`pure`][pure] [`view`][view], and `write`
///
/// For non-payable methods the [`#[public]`][public] macro can figure state mutability out for you based
/// on the types of the arguments. Functions with `&self` will be considered `view`, those with
/// `&mut self` will be considered `write`, and those with neither will be considered `pure`. Please note that
/// `pure` and `view` functions may change the state of other contracts by calling into them, or
/// even this one if the `reentrant` feature is enabled.
///
/// Please refer to the [SDK Feature Overview][overview] for more information on defining methods.
///
/// # Inheritance, `#[inherit]`, and `#[borrow]`
///
/// Composition in Rust follows that of Solidity. Types that implement [`Router`], the trait that
/// [`#[public]`][public] provides, can be connected via inheritance.
///
/// ```
/// # extern crate alloc;
/// # use alloy_primitives::U256;
/// # use stylus_proc::{entrypoint, public, storage};
/// # use stylus_sdk::prelude::*;
/// # #[entrypoint] #[storage] struct Token { #[borrow] erc20: Erc20, }
/// #[public]
/// #[inherit(Erc20)]
/// impl Token {
///     pub fn mint(&mut self, amount: U256) -> Result<(), Vec<u8>> {
///         // ...
/// #       Ok(())
///     }
/// }
///
/// #[storage] struct Erc20;
/// #[public]
/// impl Erc20 {
///     pub fn balance_of() -> Result<U256, Vec<u8>> {
///         // ...
/// #       Ok(U256::ZERO)
///     }
/// }
/// ```
///
/// Because `Token` inherits `Erc20` in the above, if `Token` has the [`#[entrypoint]`][entrypoint], calls to the
/// contract will first check if the requested method exists within `Token`. If a matching function is not found,
/// it will then try the `Erc20`. Only after trying everything `Token` inherits will the call revert.
///
/// Note that because methods are checked in that order, if both implement the same method, the one in `Token`
/// will override the one in `Erc20`, which won't be callable. This allows for patterns where the developer
/// imports a crate implementing a standard, like ERC 20, and then adds or overrides just the methods they
/// want to without modifying the imported `Erc20` type.
///
/// Stylus does not currently contain explicit `override` or `virtual` keywords for explicitly
/// marking override functions. It is important, therefore, to carefully ensure that contracts are
/// only overriding the functions.
///
/// Inheritance can also be chained. `#[inherit(Erc20, Erc721)]` will inherit both `Erc20` and `Erc721`, checking
/// for methods in that order. `Erc20` and `Erc721` may also inherit other types themselves. Method resolution
/// finds the first matching method by [`Depth First Search`][dfs].
///
/// Note that for the above to work, Token must implement [`Borrow<Erc20>`][Borrow] and
/// [`BorrowMut<Erc20>`][BorrowMut]. You can implement this yourself, but for simplicity,
/// [`#[storage]`][storage] and [`sol_storage!`][sol_storage] provide a
/// `#[borrow]` annotation.
///
/// ```
/// # extern crate alloc;
/// # use stylus_sdk::prelude::*;
/// # use stylus_proc::{entrypoint, public, sol_storage};
/// sol_storage! {
///     #[entrypoint]
///     pub struct Token {
///         #[borrow]
///         Erc20 erc20;
///     }
///
///     pub struct Erc20 {
///         uint256 total;
///     }
/// }
/// # #[public] impl Token {}
/// # #[public] impl Erc20 {}
/// ```
///
/// In the future we plan to simplify the SDK so that [`Borrow`][Borrow] isn't needed and so that
/// [`Router`] composition is more configurable. The motivation for this becomes clearer in complex
/// cases of multi-level inheritance, which we intend to improve.
///
/// # Exporting a Solidity interface
///
/// Recall that Stylus contracts are fully interoperable across all languages, including Solidity.
/// The Stylus SDK provides tools for exporting a Solidity interface for your contract so that others
/// can call it. This is usually done with the cargo stylus [CLI tool][cli].
///
/// The SDK does this automatically via a feature flag called `export-abi` that causes the
/// [`#[public]`][public] and [`#[entrypoint]`][entrypoint] macros to generate a `main` function
/// that prints the Solidity ABI to the console.
///
/// ```sh
/// cargo run --features export-abi --target <triple>
/// ```
///
/// Note that because the above actually generates a `main` function that you need to run, the target
/// can't be `wasm32-unknown-unknown` like normal. Instead you'll need to pass in your target triple,
/// which cargo stylus figures out for you. This `main` function is also why the following commonly
/// appears in the `main.rs` file of Stylus contracts.
///
/// ```no_run
/// #![cfg_attr(not(feature = "export-abi"), no_main)]
/// ```
///
/// Here's an example output. Observe that the method names change from Rust's `snake_case` to Solidity's
/// `camelCase`. For compatibility reasons, onchain method selectors are always `camelCase`. We'll provide
/// the ability to customize selectors very soon. Note too that you can use argument names like "address"
/// without fear. The SDK will prepend an `_` when necessary.
///
/// ```solidity
/// interface Erc20 {
///     function name() external pure returns (string memory);
///
///     function balanceOf(address _address) external view returns (uint256);
/// }
///
/// interface Weth is Erc20 {
///     function mint() external payable;
///
///     function burn(uint256 amount) external;
/// }
/// ```
///
/// [storage]: macro@storage
/// [sol_storage]: macro@sol_storage
/// [entrypoint]: macro@entrypoint
/// [public]: macro@public
/// [overview]: https://docs.arbitrum.io/stylus/reference/rust-sdk-guide#methods
/// [`Router`]: https://docs.rs/stylus-sdk/latest/stylus_sdk/abi/trait.Router.html
/// [Borrow]: https://doc.rust-lang.org/std/borrow/trait.Borrow.html
/// [BorrowMut]: https://doc.rust-lang.org/std/borrow/trait.BorrowMut.html
/// [value]: https://docs.rs/stylus-sdk/latest/stylus_sdk/msg/fn.value.html
/// [payable]: https://docs.alchemy.com/docs/solidity-payable-functions
/// [view]: https://docs.soliditylang.org/en/develop/contracts.html#view-functions
/// [pure]: https://docs.soliditylang.org/en/develop/contracts.html#pure-functions
/// [cli]: https://github.com/OffchainLabs/cargo-stylus#exporting-solidity-abis
/// [dfs]: https://en.wikipedia.org/wiki/Depth-first_search
#[proc_macro_attribute]
#[proc_macro_error]
pub fn public(attr: TokenStream, input: TokenStream) -> TokenStream {
    macros::public(attr, input)
}

#[doc(hidden)]
#[deprecated = "please use `#[public]` instead"]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn external(attr: TokenStream, input: TokenStream) -> TokenStream {
    public(attr, input)
}

/// Implements the AbiType for arbitrary structs, allowing them to be used in external method
/// return types and parameters. This derive is intended to be used within the
/// [alloy_sol_types::sol] macro.
///
/// ```
/// # use alloy_sol_types::sol;
/// # use stylus_proc::AbiType;
/// sol! {
///     #[derive(AbiType)]
///     struct Foo {
///         uint256 bar;
///     }
/// }
/// ```
#[proc_macro_derive(AbiType)]
#[proc_macro_error]
pub fn derive_abi_type(input: TokenStream) -> TokenStream {
    macros::derive_abi_type(input)
}
