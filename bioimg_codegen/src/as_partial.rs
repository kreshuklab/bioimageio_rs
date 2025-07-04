use quote::{quote, format_ident};
use syn::{parse_quote, parse_quote_spanned, spanned::Spanned};
use proc_macro::TokenStream;


pub struct SerdeDefaultAttrParams;

impl syn::parse::Parse for SerdeDefaultAttrParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let first_token_span = input.span();
        let default_token: syn::Ident = input.parse()?;
        if default_token.to_string() != "default" {
            return Err(syn::Error::new(first_token_span, "Expected 'default' token"))
        }
        if input.is_empty() {
            return Ok(Self)
        }
        input.parse::<syn::Token![=]>()?;
        let _default_function_name: syn::LitStr = input.parse()?;
        Ok(Self)
    }
}

trait IAttrExt{
    fn is_serde_attr(&self) -> bool;
    fn is_serde_default(&self) -> bool;
}

impl IAttrExt for syn::Attribute{
    fn is_serde_attr(&self) -> bool {
        let Some(last_segment) = self.path().segments.last() else {
            return false;
        };
        let expected: syn::PathSegment = parse_quote!(serde);
        return *last_segment == expected
    }
    fn is_serde_default(&self) -> bool {
        if !self.is_serde_attr() {
            return false
        }
        if matches!(self.style, syn::AttrStyle::Inner(_)){
            return false;
        }
        let syn::Meta::List(meta_list) = &self.meta else {
            return false;
        };
        match meta_list.parse_args::<SerdeDefaultAttrParams>() {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}


pub fn do_derive_as_partial(input: TokenStream) -> syn::Result<TokenStream>{
    // Parse the input tokens into a syntax tree.
    let input = syn::parse::<syn::ItemStruct>(input)?;
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
