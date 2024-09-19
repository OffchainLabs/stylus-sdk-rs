// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Generate implementations of [`stylus_sdk::abi::AbiType`] and all required associated traits by
//! proxying to an existing type which implements these traits.
//!
//! The type being implemented must implement `From<ProxyType>` and `Deref<Target = ProxyType>`.

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse_quote;

use crate::imports::{
    alloy_sol_types::{private::SolTypeValue, SolType, SolValue},
    stylus_sdk::abi::AbiType,
};

/// Implementations of all traits required for a [`stylus_sdk::abi::AbiType`].
#[derive(Debug)]
pub struct ImplAbiProxy {
    abi_type: syn::ItemImpl,
    sol_type: syn::ItemImpl,
    sol_value: syn::ItemImpl,
    sol_type_value: syn::ItemImpl,
}

impl ImplAbiProxy {
    /// Generate all the required implementations.
    pub fn new(self_ty: &syn::Ident, proxy_ty: &syn::Type, sol_ty: &syn::Type) -> Self {
        Self {
            abi_type: impl_abi_type(self_ty, proxy_ty),
            sol_type: impl_sol_type(self_ty, sol_ty),
            sol_value: impl_sol_value(self_ty),
            sol_type_value: impl_sol_type_value(self_ty, sol_ty),
        }
    }
}

impl ToTokens for ImplAbiProxy {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.abi_type.to_tokens(tokens);
        self.sol_type.to_tokens(tokens);
        self.sol_value.to_tokens(tokens);
        self.sol_type_value.to_tokens(tokens);
    }
}

/// Implement [`stylus_sdk::abi::AbiType`].
fn impl_abi_type(self_ty: &syn::Ident, proxy_ty: &syn::Type) -> syn::ItemImpl {
    parse_quote! {
        impl #AbiType for #self_ty {
            type SolType = #self_ty;

            const ABI: stylus_sdk::abi::ConstString = <#proxy_ty as #AbiType>::ABI;
        }
    }
}

/// Implement [`alloy_sol_types::SolType`].
fn impl_sol_type(self_ty: &syn::Ident, sol_ty: &syn::Type) -> syn::ItemImpl {
    parse_quote! {
        impl #SolType for #self_ty {
            type RustType = #self_ty;
            type Token<'a> = <#sol_ty as #SolType>::Token<'a>;

            const SOL_NAME: &'static str = <#sol_ty as #SolType>::SOL_NAME;
            const ENCODED_SIZE: Option<usize> = <#sol_ty as #SolType>::ENCODED_SIZE;
            const PACKED_ENCODED_SIZE: Option<usize> = <#sol_ty as #SolType>::PACKED_ENCODED_SIZE;

            fn valid_token(token: &Self::Token<'_>) -> bool {
                <#sol_ty as #SolType>::valid_token(token)
            }

            fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                #sol_ty::detokenize(token).into()
            }
        }
    }
}

/// Implement [`alloy_sol_types::SolValue`].
fn impl_sol_value(self_ty: &syn::Ident) -> syn::ItemImpl {
    parse_quote! {
        impl #SolValue for #self_ty {
            type SolType = #self_ty;
        }
    }
}

/// Implement [`alloy_sol_types::private::SolTypeValue`].
fn impl_sol_type_value(self_ty: &syn::Ident, sol_ty: &syn::Type) -> syn::ItemImpl {
    parse_quote! {
        impl #SolTypeValue<Self> for #self_ty {
            #[inline]
            fn stv_to_tokens(&self) -> <Self as #SolType>::Token<'_> {
                use core::ops::Deref;
                <#sol_ty as #SolType>::tokenize(self.deref())
            }

            #[inline]
            fn stv_abi_encoded_size(&self) -> usize {
                use core::ops::Deref;
                <#sol_ty as #SolType>::abi_encoded_size(self.deref())
            }

            #[inline]
            fn stv_abi_packed_encoded_size(&self) -> usize {
                use core::ops::Deref;
                <#sol_ty as #SolType>::abi_packed_encoded_size(self.deref())
            }

            #[inline]
            fn stv_eip712_data_word(&self) -> stylus_sdk::alloy_sol_types::Word {
                use core::ops::Deref;
                <#sol_ty as #SolType>::eip712_data_word(self.deref())
            }

            #[inline]
            fn stv_abi_encode_packed_to(&self, out: &mut stylus_sdk::alloy_sol_types::private::Vec<u8>) {
                use core::ops::Deref;
                <#sol_ty as #SolType>::abi_encode_packed_to(self.deref(), out)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::ImplAbiProxy;
    use crate::utils::testing::assert_ast_eq;

    #[test]
    fn test_impl_abi_proxy() {
        let proxy = ImplAbiProxy::new(&parse_quote!(Foo), &parse_quote!(Bar), &parse_quote!(Baz));
        assert_ast_eq(
            proxy.abi_type,
            parse_quote! {
                impl stylus_sdk::abi::AbiType for Foo {
                    type SolType = Foo;

                    const ABI: stylus_sdk::abi::ConstString = <Bar as stylus_sdk::abi::AbiType>::ABI;
                }
            },
        );
        assert_ast_eq(
            proxy.sol_type,
            parse_quote! {
                impl stylus_sdk::alloy_sol_types::SolType for Foo {
                    type RustType = Foo;
                    type Token<'a> = <Baz as stylus_sdk::alloy_sol_types::SolType>::Token<'a>;

                    const SOL_NAME: &'static str = <Baz as stylus_sdk::alloy_sol_types::SolType>::SOL_NAME;
                    const ENCODED_SIZE: Option<usize> = <Baz as stylus_sdk::alloy_sol_types::SolType>::ENCODED_SIZE;
                    const PACKED_ENCODED_SIZE: Option<usize> = <Baz as stylus_sdk::alloy_sol_types::SolType>::PACKED_ENCODED_SIZE;

                    fn valid_token(token: &Self::Token<'_>) -> bool {
                        <Baz as stylus_sdk::alloy_sol_types::SolType>::valid_token(token)
                    }

                    fn detokenize(token: Self::Token<'_>) -> Self::RustType {
                        Baz::detokenize(token).into()
                    }
                }
            },
        );
        assert_ast_eq(
            proxy.sol_value,
            parse_quote! {
                impl stylus_sdk::alloy_sol_types::SolValue for Foo {
                    type SolType = Foo;
                }
            },
        );
        assert_ast_eq(
            proxy.sol_type_value,
            parse_quote! {
                impl stylus_sdk::alloy_sol_types::private::SolTypeValue<Self> for Foo {
                    #[inline]
                    fn stv_to_tokens(&self) -> <Self as stylus_sdk::alloy_sol_types::SolType>::Token<'_> {
                        use core::ops::Deref;
                        <Baz as stylus_sdk::alloy_sol_types::SolType>::tokenize(self.deref())
                    }

                    #[inline]
                    fn stv_abi_encoded_size(&self) -> usize {
                        use core::ops::Deref;
                        <Baz as stylus_sdk::alloy_sol_types::SolType>::abi_encoded_size(self.deref())
                    }

                    #[inline]
                    fn stv_abi_packed_encoded_size(&self) -> usize {
                        use core::ops::Deref;
                        <Baz as stylus_sdk::alloy_sol_types::SolType>::abi_packed_encoded_size(self.deref())
                    }

                    #[inline]
                    fn stv_eip712_data_word(&self) -> stylus_sdk::alloy_sol_types::Word {
                        use core::ops::Deref;
                        <Baz as stylus_sdk::alloy_sol_types::SolType>::eip712_data_word(self.deref())
                    }

                    #[inline]
                    fn stv_abi_encode_packed_to(&self, out: &mut stylus_sdk::alloy_sol_types::private::Vec<u8>) {
                        use core::ops::Deref;
                        <Baz as stylus_sdk::alloy_sol_types::SolType>::abi_encode_packed_to(self.deref(), out)
                    }
                }
            },
        );
    }
}
