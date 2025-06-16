// Copyright 2023-2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Ident, ItemImpl, Signature, ReturnType, Type, TypePath, ImplItem,
};
use alloy_primitives::Address;

pub fn contract_client_gen(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Check if the attribute token stream is empty (no arguments passed).
    if !attr.is_empty() {
        return syn::Error::new_spanned(
            proc_macro2::TokenStream::from(attr),
            "The `contract_client_gen` macro no longer accepts attribute arguments. Provide the address to the generated client's `new()` function at runtime.",
        )
        .to_compile_error()
        .into();
    }

    // Parse the item as an `impl` block (e.g., `impl MyTrait for MyStruct`).
    let item_impl: ItemImpl = parse_macro_input!(item as ItemImpl);

    // Extract generics, the trait path, and the self type from the impl block.
    let (impl_generics, ty_generics, where_clause) = item_impl.generics.split_for_impl();
    let trait_path = item_impl.trait_.as_ref()
                                    .expect("`contract_client_gen` must be applied to an `impl` block for a trait (e.g., `impl Trait for Type`).")
                                    .1 // Get the syn::Path of the trait (e.g., `MyContractTrait`)
                                    .clone();

    // Use the last segment of the trait path for naming (e.g., `MyContractTrait` from `some_module::MyContractTrait`).
    let last_segment_ident = trait_path.segments.last()
                                            .expect("Trait path must have at least one segment")
                                            .ident
                                            .clone();
    let last_segment_ident_span = last_segment_ident.span();

    // Construct the generated struct name without a unique ID.
    // This relies on the assumption that `contract_client_gen` is called once per type_path.
    let generated_struct_name = Ident::new(
        &format!("Generated{}Client", last_segment_ident),
        last_segment_ident_span,
    );

    // Collect generated methods from the `impl` block's items.
    let generated_methods_and_items = item_impl.items.iter().filter_map(|impl_item| {
        if let ImplItem::Fn(method) = impl_item {
            let sig = &method.sig;
            let method_name = &sig.ident;
            let inputs = &sig.inputs;
            let output = &sig.output;
            let generics = &sig.generics;
            let async_token = &sig.asyncness;
            let const_token = &sig.constness;
            let unsafety_token = &sig.unsafety;
            let abi = &sig.abi;

            let default_return_value = match output {
                ReturnType::Default => quote! { () },
                ReturnType::Type(_, ty) => {
                    match &**ty {
                        Type::Path(TypePath { path, .. }) if path.is_ident("u8") => quote! { 0u8 },
                        Type::Path(TypePath { path, .. }) if path.is_ident("u16") => quote! { 0u16 },
                        Type::Path(TypePath { path, .. }) if path.is_ident("u32") => quote! { 0u32 },
                        Type::Path(TypePath { path, .. }) if path.is_ident("u64") => quote! { 0u64 },
                        Type::Path(TypePath { path, .. }) if path.is_ident("u128") => quote! { 0u128 },
                        Type::Path(TypePath { path, .. }) if path.is_ident("usize") => quote! { 0usize },
                        Type::Path(TypePath { path, .. }) if path.is_ident("i8") => quote! { 0i8 },
                        Type::Path(TypePath { path, .. }) if path.is_ident("i16") => quote! { 0i16 },
                        Type::Path(TypePath { path, .. }) if path.is_ident("i32") => quote! { 0i32 },
                        Type::Path(TypePath { path, .. }) if path.is_ident("i64") => quote! { 0i64 },
                        Type::Path(TypePath { path, .. }) if path.is_ident("i128") => quote! { 0i128 },
                        Type::Path(TypePath { path, .. }) if path.is_ident("isize") => quote! { 0isize },
                        Type::Path(TypePath { path, .. }) if path.is_ident("bool") => quote! { false },
                        Type::Path(TypePath { path, .. }) if path.is_ident("String") => quote! { String::new() },
                        Type::Path(TypePath { path, .. }) if path.is_ident("Address") => quote! { alloy_primitives::Address::ZERO },
                        Type::Path(TypePath { path, .. }) if path.is_ident("U256") => quote! { alloy_primitives::U256::ZERO },
                        Type::Tuple(syn::TypeTuple { elems, .. }) if elems.is_empty() => quote! { () },
                        _ => {
                            quote! { Default::default() }
                        }
                    }
                }
            };

            Some(quote! {
                #const_token #async_token #unsafety_token #abi fn #method_name #generics(#inputs) #output {
                    println!("(Simulated Call) Executing method: {}{}", stringify!(#method_name), stringify!(#generics));
                    #default_return_value
                }
            })
        } else {
            Some(quote! { #impl_item })
        }
    }).collect::<proc_macro2::TokenStream>();


    let output = quote! {
        // Re-emit the original `impl` block.
        #item_impl

        /// Automatically generated client for a contract associated with a specific address,
        /// implementing the functionality defined by the associated trait.
        #[derive(Debug, Clone)]
        pub struct #generated_struct_name {
            address: alloy_primitives::Address,
        }

        impl #generated_struct_name {
            /// Creates a new instance of this generated contract client.
            /// The contract address is provided at runtime.
            pub fn new(address: alloy_primitives::Address) -> Self {
                #generated_struct_name {
                    address,
                }
            }

            /// Returns the blockchain address of this contract instance.
            pub fn address(&self) -> alloy_primitives::Address {
                self.address
            }

            // Generated methods are placed directly into the `impl GeneratedClient { ... }` block.
            #generated_methods_and_items
        }

        // Also implement the trait for the generated client.
        impl #impl_generics #trait_path for #generated_struct_name #ty_generics #where_clause {
            #generated_methods_and_items
        }
    };

    output.into()
}
