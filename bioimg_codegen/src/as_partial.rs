
use quote::{quote, quote_spanned, format_ident};
use syn::spanned::Spanned;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;


pub fn do_derive_as_partial(input: TokenStream) -> syn::Result<TokenStream>{
    // Parse the input tokens into a syntax tree.
    let input = syn::parse::<syn::ItemStruct>(input)?;
    let struct_name = &input.ident;
    let struct_vis = input.vis;
    let partial_struct_name = format_ident!("Partial{}", struct_name);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let where_predicates = where_clause.map(|wc| wc.predicates.clone());

    let generics_bound_to_AsPartial: Vec<TokenStream2> = input.generics.type_params()
        .map(|gen_ty| {
            let gen_ty_ident = &gen_ty.ident;
            quote! { #gen_ty_ident : ::bioimg_spec::util::AsPartial }
        })
        .collect();

    let generics_bound_to_serde: Vec<TokenStream2> = input.generics.type_params()
        .map(|gen_ty| {
            let gen_ty_ident = &gen_ty.ident;
            quote! { #gen_ty_ident : ::serde::Serialize + ::serde::de::DeserializeOwned }
        })
        .collect();

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

    let expanded = quote! {
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        #struct_vis struct #partial_struct_name #ty_generics
        where
            #(#generics_bound_to_AsPartial,)*
            #where_predicates
        {
            #(#partial_fields)*
        }

        impl #impl_generics ::bioimg_spec::util::AsPartial for #struct_name #ty_generics
        where
            #(#generics_bound_to_AsPartial,)*
            #(#generics_bound_to_serde,)*
            #where_predicates
            // FIXME: extra where clauses... i guess enforce that fields are AsPartial?
        {
            type Partial = #partial_struct_name #impl_generics;
        }
    };

    Ok(proc_macro::TokenStream::from(expanded))
}
