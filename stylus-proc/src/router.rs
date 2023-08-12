use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, token, Expr, LitStr, Path, Token};

#[derive(Clone, Debug)]
pub struct ItemRoute {
    // pub method
    pub prefix: LitStr,
    // pub middleware
    pub fat_arrow_token: Token![=>],
    pub handler: ItemHandler,
}

impl Parse for ItemRoute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item_route = ItemRoute {
            prefix: input.parse()?,
            fat_arrow_token: input.parse()?,
            handler: input.parse()?,
        };

        Ok(item_route)
    }
}

#[derive(Clone, Debug)]
pub enum ItemHandler {
    Expr(Box<Expr>),
}

impl Parse for ItemHandler {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        let _: Path = fork.parse()?;
        Ok(ItemHandler::Expr(input.parse()?))
    }
}

#[derive(Clone, Debug)]
pub struct Router {
    // middleware: Option<ItemWithMiddleware>,
    routes: Vec<ItemRoute>,
    // catch_all: Option<ItemCatchAll>,
}

impl Router {
    pub fn expand(&self) -> TokenStream {
        quote! {



          let expansion = "Yaaaassss".to_string();
          debug::println("in expansion");

          // self.routes has everything we need to expand

          //match selector {
            // expand each route
            // two sides of fat arrow: prefixes and handler on the right side
            // compute the selector for each prefix by keccak256(prefix += handler signature)
            // generate left side of fat arrow arm, then invoke handler and pass calldata through
          //}
        }
    }
}

impl Parse for Router {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // let middleware...

        let mut routes: Vec<ItemRoute> = Vec::new();
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
