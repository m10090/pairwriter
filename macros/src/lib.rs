use proc_macro::TokenStream;
use syn::{parse_macro_input, token::Trait, Ident, ItemImpl, ItemTrait};
use quote::quote;

#[proc_macro_attribute]
pub fn conditional_pub(attr:TokenStream,i: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as Ident);
    let input= parse_macro_input!(i as ItemTrait);
    TokenStream::from(quote! {
        #[cfg(#attr)]
        pub #input

        #[cfg(not(#attr))]
        #input
    })

}
