#![allow(incomplete_features)]
// #![feature(proc_macro_diagnostic, adt_const_params)]
#![allow(non_snake_case)]

use proc_macro::TokenStream;

mod str_marker;
mod syn_extensions;
mod restore;
mod as_partial;

////////////////////////////////////////////

#[proc_macro_derive(StrMarker, attributes(strmarker))]
pub fn derive_str_marker(input: TokenStream) -> TokenStream {
    match str_marker::do_derive_str_marker(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Restore, attributes(restore_default, restore_on_update))]
pub fn derive_restore(input: TokenStream) -> TokenStream {
    match restore::do_derive_restore(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(AsPartial)]
pub fn derive_as_partial(input: TokenStream) -> TokenStream {
    match as_partial::do_derive_as_partial(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}
