#[macro_use]
extern crate quote;

mod openapi_attr;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn query(
    args: TokenStream,
    mut input: TokenStream,
) -> TokenStream {
    input.extend(
        openapi_attr::parse_query(args.into(), input.clone().into()),
    );

    input
}

#[proc_macro_attribute]
pub fn handler(
    args: TokenStream,
    mut input: TokenStream,
) -> TokenStream {
    input.extend(
        openapi_attr::parse_handler(args.into(), input.clone().into()),
    );

    input
}