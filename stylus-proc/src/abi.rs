use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

pub fn derive_abi_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let name = &input.ident;
    let name_str = name.to_string();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics stylus_sdk::abi::AbiType for #name #ty_generics #where_clause {
            type SolType = Self;
            const ABI: stylus_sdk::abi::ConstString = stylus_sdk::abi::ConstString::new(#name_str);
        }
    }
    .into()
}
