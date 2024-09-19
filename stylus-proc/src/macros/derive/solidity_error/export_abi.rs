// Copyright 2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use syn::parse_quote;

use super::{DeriveSolidityError, SolidityErrorExtension};

#[derive(Debug, Default)]
pub struct InnerTypesExtension {
    errors: Vec<syn::Field>,
}

impl SolidityErrorExtension for InnerTypesExtension {
    type Ast = syn::ItemImpl;

    fn add_variant(&mut self, field: syn::Field) {
        self.errors.push(field);
    }

    fn codegen(err: &DeriveSolidityError<Self>) -> syn::ItemImpl {
        let name = &err.name;
        let errors = err._ext.errors.iter();
        parse_quote! {
            impl stylus_sdk::abi::export::internal::InnerTypes for #name {
                fn inner_types() -> alloc::vec::Vec<stylus_sdk::abi::export::internal::InnerType> {
                    use alloc::{format, vec};
                    use core::any::TypeId;
                    use stylus_sdk::abi::export::internal::InnerType;
                    use stylus_sdk::alloy_sol_types::SolError;

                    vec![
                        #(
                            InnerType {
                                name: format!("error {};", <#errors as SolError>::SIGNATURE.replace(',', ", ")),
                                id: TypeId::of::<#errors>(),
                            }
                        ),*
                    ]
                }
            }
        }
    }
}
