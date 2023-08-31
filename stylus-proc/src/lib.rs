// Copyright 2022-2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

use proc_macro::TokenStream;

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

mod calls;
mod methods;
mod storage;
mod types;

#[proc_macro_attribute]
pub fn solidity_storage(attr: TokenStream, input: TokenStream) -> TokenStream {
    storage::solidity_storage(attr, input)
}

#[proc_macro]
pub fn sol_storage(input: TokenStream) -> TokenStream {
    storage::sol_storage(input)
}

#[proc_macro]
pub fn sol_interface(input: TokenStream) -> TokenStream {
    calls::sol_interface(input)
}

#[proc_macro_derive(Erase)]
pub fn derive_erase(input: TokenStream) -> TokenStream {
    storage::derive_erase(input)
}

/// For structs, this macro generates a richly-typed entrypoint that parses incoming calldata.
/// For functions, this macro generates a simple, untyped entrypoint that's bytes-in, bytes-out.
///
/// Reentrancy is disabled by default, which will cause the program to revert in cases of nested calls.
/// This behavior can be overridden by passing `#[entrypoint(allow_reentrancy = true)]`
#[proc_macro_attribute]
pub fn entrypoint(attr: TokenStream, input: TokenStream) -> TokenStream {
    methods::entrypoint::entrypoint(attr, input)
}

#[proc_macro_attribute]
pub fn external(attr: TokenStream, input: TokenStream) -> TokenStream {
    methods::external::external(attr, input)
}
