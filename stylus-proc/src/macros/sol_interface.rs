// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_quote};
use syn_solidity::{visit, Spanned, Visit};

use crate::{
    impls::abi_proxy::ImplAbiProxy,
    imports::{
        alloy_primitives::Address as AlloyAddress,
        alloy_sol_types::{sol_data::Address as SolAddress, SolType},
    },
    types::{Purity, SolidityTypeInfo},
    utils::build_selector,
};

/// Implementation of the [`sol_interface!`][crate::sol_interface] macro.
///
/// This implementation uses [`SolInterfaceVisitor`] which implements [`syn_solidity::Visit`] to
/// collect interface declarations and convert them to Rust structs with a method for each of the
/// interface's functions.
pub fn sol_interface(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let file = parse_macro_input!(input as syn_solidity::File);
    SolInterfaceVisitor::from(&file).into_token_stream().into()
}

/// Visitor for the [`sol_interface!`][crate::sol_interface] macro.
///
/// Collects all defined interfaces, doing error checking along the way.
#[derive(Debug, Default)]
struct SolInterfaceVisitor {
    interfaces: Vec<Interface>,
}

impl From<&syn_solidity::File> for SolInterfaceVisitor {
    fn from(file: &syn_solidity::File) -> Self {
        let mut visitor = Self::default();
        visitor.visit_file(file);
        visitor
    }
}

impl ToTokens for SolInterfaceVisitor {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for iface in &self.interfaces {
            iface.to_tokens(tokens);
        }
    }
}

impl<'ast> Visit<'ast> for SolInterfaceVisitor {
    fn visit_file(&mut self, file: &'ast syn_solidity::File) {
        for attr in &file.attrs {
            emit_error!(attr, "attribute not supported");
        }
        visit::visit_file(self, file);
    }

    fn visit_item(&mut self, item: &'ast syn_solidity::Item) {
        if !matches!(item, syn_solidity::Item::Contract(..)) {
            emit_error!(item.span(), "not an interface");
        }
        visit::visit_item(self, item);
    }

    fn visit_item_contract(&mut self, contract: &'ast syn_solidity::ItemContract) {
        if !contract.is_interface() {
            emit_error!(contract.span(), "not an interface");
            return;
        }

        if let Some(inheritance) = &contract.inheritance {
            emit_error!(inheritance.span(), "inheritance not supported");
        }

        self.interfaces.push(Interface::from(contract));
    }
}

/// Interface defined in the [`sol_interface!`][crate::sol_interface] macro.
#[derive(Debug)]
struct Interface {
    item_struct: syn::ItemStruct,
    item_impl: syn::ItemImpl,
    impl_deref: syn::ItemImpl,
    impl_from_address: syn::ItemImpl,
    impl_abi_proxy: ImplAbiProxy,
}

impl Interface {
    /// Add a function to the interface definition.
    fn add_function(
        &mut self,
        attrs: &[syn::Attribute],
        name: &syn_solidity::SolIdent,
        purity: Purity,
        params: FunctionParameters,
        return_type: syn::Type,
    ) {
        let rust_name = syn::Ident::new(&name.to_string().to_case(Case::Snake), name.span());

        // build selector
        let selector = build_selector(name, params.params.iter().map(|p| &p.type_info.sol_type));
        let [selector0, selector1, selector2, selector3] = selector;

        // determine which context and kind of call to use
        let (context, call) = match purity {
            Purity::Pure | Purity::View => (
                quote!(stylus_sdk::stylus_core::calls::StaticCallContext),
                quote!(stylus_sdk::call::static_call),
            ),
            Purity::Write => (
                quote!(stylus_sdk::stylus_core::calls::NonPayableCallContext),
                quote!(stylus_sdk::call::call),
            ),
            Purity::Payable => (
                quote!(stylus_sdk::stylus_core::calls::MutatingCallContext),
                quote!(stylus_sdk::call::call),
            ),
        };

        let sol_args = params
            .params
            .iter()
            .map(|param| param.type_info.alloy_type.clone());
        let rust_args = params.params.iter().map(|param| -> syn::FnArg {
            let FunctionParameter {
                name,
                type_info: SolidityTypeInfo { alloy_type, .. },
                ..
            } = param;
            parse_quote!(#name: <#alloy_type as #SolType>::RustType)
        });
        let rust_arg_names = params.params.iter().map(|param| param.name.clone());

        self.item_impl.items.push(parse_quote! {
            #(#attrs)*
            #[allow(deprecated)]
            pub fn #rust_name(&self, host: &dyn stylus_sdk::stylus_core::host::Host, context: impl #context #(, #rust_args)*) ->
                Result<<#return_type as #SolType>::RustType, stylus_sdk::stylus_core::calls::errors::Error>
            {
                let args = <(#(#sol_args,)*) as #SolType>::abi_encode_params(&(#(#rust_arg_names,)*));
                let mut calldata = vec![#selector0, #selector1, #selector2, #selector3];
                calldata.extend(args);
                let returned = #call(host, context, self.address, &calldata)?;
                Ok(<(#return_type,) as #SolType>::abi_decode_params(&returned)?.0)
            }
        });
    }
}

impl From<&syn_solidity::ItemContract> for Interface {
    fn from(contract: &syn_solidity::ItemContract) -> Self {
        let name = contract.name.clone().into();
        let attrs = &contract.attrs;

        let mut iface = Self {
            item_struct: parse_quote! {
                #(#attrs)*
                pub struct #name {
                    pub address: #AlloyAddress,
                }
            },
            item_impl: parse_quote! {
                impl #name {
                    pub fn new(address: #AlloyAddress) -> Self {
                        Self { address }
                    }
                }
            },
            impl_deref: parse_quote! {
                impl core::ops::Deref for #name {
                    type Target = #AlloyAddress;

                    fn deref(&self) -> &Self::Target {
                        &self.address
                    }
                }
            },
            impl_from_address: parse_quote! {
                impl From<#AlloyAddress> for #name {
                    fn from(address: #AlloyAddress) -> Self {
                        Self::new(address)
                    }
                }
            },
            impl_abi_proxy: ImplAbiProxy::new(
                &name,
                &AlloyAddress.as_type(),
                &SolAddress.as_type(),
            ),
        };

        iface.visit_item_contract(contract);
        iface
    }
}

impl ToTokens for Interface {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.item_struct.to_tokens(tokens);
        self.item_impl.to_tokens(tokens);
        self.impl_deref.to_tokens(tokens);
        self.impl_from_address.to_tokens(tokens);
        self.impl_abi_proxy.to_tokens(tokens);
    }
}

impl<'ast> Visit<'ast> for Interface {
    fn visit_item(&mut self, item: &'ast syn_solidity::Item) {
        if !matches!(item, syn_solidity::Item::Function(_)) {
            emit_error!(item.span(), "unsupported interface item");
        }
        visit::visit_item(self, item);
    }

    fn visit_item_function(&mut self, function: &'ast syn_solidity::ItemFunction) {
        if !matches!(function.kind, syn_solidity::FunctionKind::Function(_)) {
            emit_error!(function.span(), "unsupported function type");
        }

        let Some(name) = &function.name else {
            emit_error!(function.span(), "function has no name");
            return;
        };

        // determine the purity and check for external attribute
        let mut purity = None;
        let mut external = false;
        for attr in &function.attributes.0 {
            match attr {
                syn_solidity::FunctionAttribute::Mutability(mutability) => {
                    if purity.is_some() {
                        emit_error!(attr.span(), "more than one purity attribute specified");
                        continue;
                    }
                    purity = Some(match mutability {
                        syn_solidity::Mutability::Pure(_) => Purity::Pure,
                        syn_solidity::Mutability::View(_) => Purity::View,
                        syn_solidity::Mutability::Payable(_) => Purity::Payable,
                        syn_solidity::Mutability::Constant(_) => {
                            emit_error!(
                                mutability.span(),
                                "constant mutibility no longer supported"
                            );
                            continue;
                        }
                    });
                }
                syn_solidity::FunctionAttribute::Visibility(vis) => match vis {
                    syn_solidity::Visibility::External(_) => external = true,
                    _ => {
                        emit_error!(vis.span(), "visibility must be external");
                    }
                },
                _ => emit_error!(attr.span(), "unsupported function attribute"),
            }
        }
        let purity = purity.unwrap_or(Purity::Write);
        if !external {
            emit_error!(function.span(), "visibility must be external");
        }

        // build the parameter list
        let mut params = FunctionParameters::new();
        params.visit_parameter_list(&function.parameters);

        // get the return type
        let return_type = match function.return_type() {
            Some(ty) => SolidityTypeInfo::from(&ty).alloy_type,
            None => parse_quote!(()),
        };

        self.add_function(&function.attrs, name, purity, params, return_type);
    }
}

struct FunctionParameters {
    params: Vec<FunctionParameter>,
}

impl FunctionParameters {
    fn new() -> Self {
        Self { params: Vec::new() }
    }
}

impl Visit<'_> for FunctionParameters {
    fn visit_variable_declaration(&mut self, var: &syn_solidity::VariableDeclaration) {
        let type_info = SolidityTypeInfo::from(&var.ty);
        let name = match &var.name {
            Some(name) => name.clone().into(),
            None => syn::Ident::new(&format!("__argument_{}", self.params.len()), var.span()),
        };
        self.params.push(FunctionParameter { name, type_info });
    }
}

struct FunctionParameter {
    name: syn::Ident,
    type_info: SolidityTypeInfo,
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse_quote;

    use super::SolInterfaceVisitor;
    use crate::utils::testing::assert_ast_eq;

    #[test]
    fn test_sol_interface() {
        let file = syn_solidity::parse2(quote! {
            #[interface_attr]
            interface IService {
                #[function_attr]
                function makePayment(address user) payable external returns (string);
                function getConstant() pure external returns (bytes32);
                function getFoo() pure external returns (inner.Foo);
            }

            interface ITree {
                // Define more interface methods here
            }
        })
        .unwrap();
        let visitor = SolInterfaceVisitor::from(&file);
        assert_ast_eq(
            &visitor.interfaces[0].item_struct,
            &parse_quote! {
                #[interface_attr]
                pub struct IService {
                    pub address: stylus_sdk::alloy_primitives::Address,
                }
            },
        );
        assert_ast_eq(
            &visitor.interfaces[0].item_impl,
            &parse_quote! {
                impl IService {
                    pub fn new(address: stylus_sdk::alloy_primitives::Address) -> Self {
                        Self { address }
                    }

                    #[function_attr]
                    #[allow(deprecated)]
                    pub fn make_payment(
                        &self,
                        host: &dyn stylus_sdk::stylus_core::host::Host,
                        context: impl stylus_sdk::stylus_core::calls::MutatingCallContext,
                        user: <stylus_sdk::alloy_sol_types::sol_data::Address as stylus_sdk::alloy_sol_types::SolType>::RustType,
                    ) ->
                        Result<<stylus_sdk::alloy_sol_types::sol_data::String as stylus_sdk::alloy_sol_types::SolType>::RustType, stylus_sdk::stylus_core::calls::errors::Error>
                    {
                        let args = <(
                            stylus_sdk::alloy_sol_types::sol_data::Address,
                        ) as stylus_sdk::alloy_sol_types::SolType>::abi_encode_params(&(user,));
                        let mut calldata = vec![48u8, 11u8, 228u8, 252u8];
                        calldata.extend(args);
                        let returned = stylus_sdk::call::call(host, context, self.address, &calldata)?;
                        Ok(<(
                            stylus_sdk::alloy_sol_types::sol_data::String,
                        ) as stylus_sdk::alloy_sol_types::SolType>::abi_decode_params(&returned)?.0)
                    }

                    #[allow(deprecated)]
                    pub fn get_constant(
                        &self,
                        host: &dyn stylus_sdk::stylus_core::host::Host,
                        context: impl stylus_sdk::stylus_core::calls::StaticCallContext,
                    ) ->
                        Result<<stylus_sdk::alloy_sol_types::sol_data::FixedBytes<32> as stylus_sdk::alloy_sol_types::SolType>::RustType, stylus_sdk::stylus_core::calls::errors::Error>
                    {
                        let args = <() as stylus_sdk::alloy_sol_types::SolType>::abi_encode_params(&());
                        let mut calldata = vec![241u8, 58u8, 56u8, 166u8];
                        calldata.extend(args);
                        let returned = stylus_sdk::call::static_call(host, context, self.address, &calldata)?;
                        Ok(<(stylus_sdk::alloy_sol_types::sol_data::FixedBytes<32>,) as stylus_sdk::alloy_sol_types::SolType>::abi_decode_params(&returned)?.0)
                    }

                    #[allow(deprecated)]
                    pub fn get_foo(
                        &self,
                        host: &dyn stylus_sdk::stylus_core::host::Host,
                        context: impl stylus_sdk::stylus_core::calls::StaticCallContext,
                    ) ->
                        Result<<inner::Foo as stylus_sdk::alloy_sol_types::SolType>::RustType, stylus_sdk::stylus_core::calls::errors::Error>
                    {
                        let args = <() as stylus_sdk::alloy_sol_types::SolType>::abi_encode_params(&());
                        let mut calldata = vec![36u8, 61u8, 200u8, 218u8];
                        calldata.extend(args);
                        let returned = stylus_sdk::call::static_call(host, context, self.address, &calldata)?;
                        Ok(<(inner::Foo,) as stylus_sdk::alloy_sol_types::SolType>::abi_decode_params(&returned)?.0)
                    }
                }
            },
        );
        assert_ast_eq(
            &visitor.interfaces[0].impl_deref,
            &parse_quote! {
                impl core::ops::Deref for IService {
                    type Target = stylus_sdk::alloy_primitives::Address;

                    fn deref(&self) -> &Self::Target {
                        &self.address
                    }
                }
            },
        );
        assert_ast_eq(
            &visitor.interfaces[0].impl_from_address,
            &parse_quote! {
                impl From<stylus_sdk::alloy_primitives::Address> for IService {
                    fn from(address: stylus_sdk::alloy_primitives::Address) -> Self {
                        Self::new(address)
                    }
                }
            },
        );
    }
}
