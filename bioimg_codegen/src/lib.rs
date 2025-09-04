#![allow(incomplete_features)]
#![allow(non_snake_case)]

use proc_macro::TokenStream;

mod syn_extensions;
mod restore;
mod serde_attributes;

#[proc_macro_derive(Restore, attributes(restore))]
pub fn derive_restore(input: TokenStream) -> TokenStream {
    match restore::do_derive_restore(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}
