mod response_key;

use proc_macro::TokenStream;

#[proc_macro_derive(ResponseKey, attributes(response_key))]
pub fn response_key_derive(input: TokenStream) -> TokenStream {
    response_key::response_key_derive(input)
}
