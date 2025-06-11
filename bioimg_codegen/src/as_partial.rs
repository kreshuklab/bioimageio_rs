
use quote::{quote, quote_spanned, format_ident};
use syn::spanned::Spanned;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;


pub fn do_derive_as_partial(input: TokenStream) -> syn::Result<TokenStream>{
    // Parse the input tokens into a syntax tree.
    let input = syn::parse::<syn::ItemStruct>(input)?;
    let struct_name = &input.ident;
    let raw_data_struct_name = format_ident!("Partial{}", struct_name);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let where_clauses = where_clause.map(|wc| wc.predicates.clone());

    let partial_fields: Vec::<TokenStream2> = input.fields.iter().enumerate().map(|(field_idx, field)| {
        let ident = field.ident.as_ref().map(|id| quote!(#id)).unwrap_or(quote!(#field_idx));
        let field_ty = &field.ty;
        let ident_span = ident.span();
        quote_spanned! {ident_span=>
            #[serde(default)]
            #ident: Option< <#field_ty as ::bioimg_spec::util::AsPartial>::Partial >,
        }
    })
    .collect();

    let extra_where_clauses: Vec<TokenStream2> = input.generics.type_params()
        .map(|gen_ty| {
            let gen_ty_ident = &gen_ty.ident;
            quote! { #gen_ty_ident : ::serde::Serialize + ::serde::de::DeserializeOwned + crate::util::AsPartial } //FIXME: don't use 'crate'
        })
        .collect();

    let expanded = quote! {
        // #[derive(::serde::Serialize, ::serde::Deserialize)]
        // pub struct #raw_data_struct_name #ty_generics
        // where
        //     #where_clause
        //     #(#extra_where_clauses)*
        // {
        //     #(#partial_fields)*
        // }

        //FIXME: use ::bioimg_spec instead of crate and maybe use the extern crate self as bioimg_spec trick
        impl #impl_generics crate::util::AsPartial for #struct_name #ty_generics
        where
            #(#extra_where_clauses,)*
            #where_clauses
            // FIXME: extra where clauses... i guess enforce that fields are AsPartial?
        {
            // type Partial = #raw_data_struct_name::<impl_generics>;
            type Partial = String;
        }
    };

    std::fs::write("/tmp/ty_generics.rs", quote!(#ty_generics).to_string()).unwrap();
    std::fs::write("/tmp/bla.rs", expanded.to_string()).unwrap();

    Ok(proc_macro::TokenStream::from(expanded))
}
