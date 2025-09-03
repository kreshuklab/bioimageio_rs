#![allow(incomplete_features)]
// #![feature(proc_macro_diagnostic, adt_const_params)]
#![allow(non_snake_case)]

use proc_macro::TokenStream;

mod str_marker;
mod syn_extensions;
mod restore;
mod serde_attributes;

////////////////////////////////////////////

#[proc_macro_derive(StrMarker, attributes(strmarker))]
pub fn derive_str_marker(input: TokenStream) -> TokenStream {
    match str_marker::do_derive_str_marker(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Restore, attributes(restore))]
pub fn derive_restore(input: TokenStream) -> TokenStream {
    match restore::do_derive_restore(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}
