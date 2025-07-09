use quote::{quote, format_ident};
use syn::{parse_quote, parse_quote_spanned, spanned::Spanned};
use proc_macro::TokenStream;

use crate::{serde_attributes::SerdeEnumTagParams, syn_extensions::{IAttrExt, IVariantExt}};

pub fn make_partial_enum(input: syn::ItemEnum) -> syn::Result<TokenStream>{
    let tag_params = input.attrs.iter().find_map(SerdeEnumTagParams::try_from_attr);
    let partial_struct_ident = format_ident!("Partial{}", input.ident);
    let partial_struct_fields: Vec<syn::Field> = input.variants.iter()
        .map(|v| v.to_partial_field())
        .collect::<syn::Result<_>>()?;

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let partial_struct: syn::ItemStruct = parse_quote!{
        #[derive(Clone, Debug, ::serde::Deserialize)]
        #[serde(try_from = "::serde_json::Value")]
        pub struct #partial_struct_ident #impl_generics
        #where_clause
        {
            #(#partial_struct_fields),*
        }
    };

    let impl__TryFromfrom_jsonValue: syn::ItemImpl = {
    }
}

pub fn make_partial_struct(input: syn::ItemStruct) -> syn::Result<TokenStream>{
    let struct_name = &input.ident;
    let partial_struct = {
        let mut partial_struct = input.clone();
        // partial_struct.attrs.retain(|attr| attr.is_serde_attr());
        partial_struct.ident = format_ident!("Partial{struct_name}");
        partial_struct.attrs = vec![
            parse_quote!(#[derive(::serde::Serialize, ::serde::Deserialize)]),
            parse_quote!(#[serde(bound = "")]),
        ];
        partial_struct.generics.where_clause = {
            let mut wc = partial_struct.generics.where_clause.unwrap_or(parse_quote!(where));
            let comma = parse_quote!(,);

            if !wc.predicates.empty_or_trailing() {
                wc.predicates.push_punct(comma);
            }
            for field in partial_struct.fields.iter_mut() {
                let field_ty = &field.ty;
                let span = field_ty.span();
                wc.predicates.push_value(parse_quote_spanned!{ span=>
                    #field_ty: ::bioimg_spec::util::AsSerializablePartial<Partial: std::clone::Clone + std::fmt::Debug>
                });
                wc.predicates.push_punct(comma);

                field.attrs = std::mem::take(&mut field.attrs).into_iter()
                    .filter(|attr| {
                        attr.is_serde_attr()
                    })
                    .collect();
                if !field.attrs.iter().any(|a| a.is_serde_default()){
                    field.attrs.push(parse_quote!(#[serde(default)]));
                    field.ty = parse_quote!(Option< <#field_ty as ::bioimg_spec::util::AsPartial>::Partial >);
                }
            }
            Some(wc)
        };
        partial_struct
    };
    let partial_struct_name = &partial_struct.ident;

    let (impl_generics, ty_generics, where_clause) = partial_struct.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics ::bioimg_spec::util::AsPartial for #struct_name #ty_generics
            #where_clause
        {
            type Partial = #partial_struct_name #impl_generics;
        }

        impl #impl_generics ::bioimg_spec::util::AsPartial for #partial_struct_name #ty_generics
            #where_clause
        {
            type Partial = Self;
        }

        #[derive(Clone, Debug)]
        #partial_struct
    };

    Ok(proc_macro::TokenStream::from(expanded))
}

pub fn do_derive_as_partial(input: TokenStream) -> syn::Result<TokenStream>{
    // Parse the input tokens into a syntax tree.
    let input = syn::parse::<syn::DeriveInput>(input)?;
    let struct_name = &input.ident;

    let partial_struct = {
        let mut partial_struct = input.clone();
        // partial_struct.attrs.retain(|attr| attr.is_serde_attr());
        partial_struct.ident = format_ident!("Partial{struct_name}");
        partial_struct.attrs = vec![
            parse_quote!(#[derive(::serde::Serialize, ::serde::Deserialize)]),
            parse_quote!(#[serde(bound = "")]),
        ];
        partial_struct.generics.where_clause = {
            let mut wc = partial_struct.generics.where_clause.unwrap_or(parse_quote!(where));
            let comma = parse_quote!(,);

            if !wc.predicates.empty_or_trailing() {
                wc.predicates.push_punct(comma);
            }
            for field in partial_struct.fields.iter_mut() {
                let field_ty = &field.ty;
                let span = field_ty.span();
                wc.predicates.push_value(parse_quote_spanned!{ span=>
                    #field_ty: ::bioimg_spec::util::AsSerializablePartial<Partial: std::clone::Clone + std::fmt::Debug>
                });
                wc.predicates.push_punct(comma);

                field.attrs = std::mem::take(&mut field.attrs).into_iter()
                    .filter(|attr| {
                        attr.is_serde_attr()
                    })
                    .collect();
                if !field.attrs.iter().any(|a| a.is_serde_default()){
                    field.attrs.push(parse_quote!(#[serde(default)]));
                    field.ty = parse_quote!(Option< <#field_ty as ::bioimg_spec::util::AsPartial>::Partial >);
                }
            }
            Some(wc)
        };
        partial_struct
    };
    let partial_struct_name = &partial_struct.ident;

    let (impl_generics, ty_generics, where_clause) = partial_struct.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics ::bioimg_spec::util::AsPartial for #struct_name #ty_generics
            #where_clause
        {
            type Partial = #partial_struct_name #impl_generics;
        }

        impl #impl_generics ::bioimg_spec::util::AsPartial for #partial_struct_name #ty_generics
            #where_clause
        {
            type Partial = Self;
        }

        #[derive(Clone, Debug)]
        #partial_struct
    };

    // std::fs::write(format!("/tmp/blas__{}.rs", struct_name.to_string()), expanded.to_string()).unwrap();

    Ok(proc_macro::TokenStream::from(expanded))
}
