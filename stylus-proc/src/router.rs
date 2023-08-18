use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, token, Expr, Ident, LitStr, Path, Token};

use crate::handler::{
    calldata_sig_name_template, calldata_type_template, generated_handler_name_template,
};

/**
 * WIP: The design of the router will allow for states and middleware to be associated with contexts like so:
 * This allows structure of the routes to be viewed comprehensively rather than defining things like selectors
 * and middleware on individual handlers via macro attributes, can get a visual handle on control flow
 * and state dependencies
 *
 * router! {
 *     with_state(StateStruct);
 *
 *     "balance_of" | "balanceOf" => balance_of_handler,
 *     "transfer" => transfer_handler,
 *
 *     { // defines an isolated scope for state and middleware to share between handlers
 *        // uses OwnerState as ctx.state
 *        with_state(OwnerState);
 *        with_middleware [only_owner]; //optionally use this syntax to include middlware for all routes in this scope
 *
 *        // can also use inline middleware on a individual route basis
 *        "transfer_ownership" | "transferOwnership" => with [only_owner] => transfer_ownership_handler,
 *     }
 * }
 *  */

#[derive(Clone, Debug)]
pub struct RouteParser {
    pub prefix: LitStr,
    // TODO: pub middleware
    pub fat_arrow_token: Token![=>],
    pub handler: Ident,
}

impl Parse for RouteParser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item_route = RouteParser {
            prefix: input.parse()?,
            fat_arrow_token: input.parse()?,
            handler: input.parse()?,
        };

        Ok(item_route)
    }
}

#[derive(Clone, Debug)]
pub struct RouterParser {
    // TODO: middleware: Option<ItemWithMiddleware>,
    routes: Vec<RouteParser>,
    // TODO: catch_all: Option<ItemCatchAll>,
}

impl Parse for RouterParser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // TODO: let middleware...

        let mut routes: Vec<RouteParser> = Vec::new();
        while input.peek(LitStr) {
            routes.push(input.parse()?);

            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        // let catch_all = input.peek(Token![_]).then(|| input.parse()).transpose()?;

        Ok(Self { routes })
    }
}

// expand each route
// two sides of fat arrow: prefixes and handler on the right side
// compute the selector for each prefix by keccak256(prefix += handler signature)
// generate left side of fat arrow arm, then invoke handler and pass calldata through
impl RouterParser {
    pub fn expand(&self) -> TokenStream {
        let calldata_sig_idents: Vec<TokenStream> = self
            .routes
            .clone()
            .into_iter()
            .map(|parsed_route| {
                let calldata_sig_const =
                    format_ident!(calldata_sig_name_template!(), parsed_route.prefix.value());
                quote! {
                   #calldata_sig_const
                }
            })
            .collect();

        println!("{:?}", calldata_sig_idents);

        let calldata_type_idents: Vec<TokenStream> = self
            .routes
            .clone()
            .into_iter()
            .map(|parsed_route| {
                let calldata_type_ident =
                    format_ident!(calldata_type_template!(), parsed_route.handler);
                quote! { #calldata_type_ident }
            })
            .collect();

        println!("calldata_type_idents: {:?}", calldata_type_idents);

        let calldata_match_arms: Vec<TokenStream> = self
            .routes
            .clone()
            .into_iter()
            .map(|parsed_route| {
                let calldata_sig_name = format_ident!(
                    calldata_sig_name_template!(),
                    parsed_route.handler.to_string().to_uppercase()
                );
                let generated_handler_name =
            format_ident!(generated_handler_name_template!(), parsed_route.handler.to_string());
                
                quote! { #calldata_sig_name::CAMEL_SELECTOR | #calldata_sig_name::SNAKE_SELECTOR => { #generated_handler_name(data) }}
            })
            .collect();

        println!("calldata_match_patterns: {:?}", calldata_match_arms);

        quote! {
          use stylus_sdk::router::extract_call_parts;
          let (selector, data) = extract_call_parts(input);

          match selector {
            #(#calldata_match_arms ,)*
            _ => {
              debug::println("Default Selector");
              return Ok(vec![])
            }
          }
        }
    }
}
