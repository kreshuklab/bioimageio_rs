use quote::{quote, format_ident};
use syn::{parse_quote, parse_quote_spanned, spanned::Spanned};
use proc_macro::TokenStream;


pub fn do_derive_as_partial(input: TokenStream) -> syn::Result<TokenStream>{
    // Parse the input tokens into a syntax tree.
    let input = syn::parse::<syn::ItemStruct>(input)?;
    let struct_name = &input.ident;

    let partial_struct = {
        let mut partial_struct = input.clone();
        partial_struct.ident = format_ident!("Partial{struct_name}");
        partial_struct.attrs.push(parse_quote!(#[derive(::serde::Serialize, ::serde::Deserialize)]));
        partial_struct.attrs.push(parse_quote!(#[serde(bound = "")]));
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
                    #field_ty: ::bioimg_spec::util::AsSerializablePartial
                });
                wc.predicates.push_punct(comma);

                field.attrs = vec![parse_quote!(#[serde(default)])];
                field.ty = parse_quote!(Option< <#field_ty as ::bioimg_spec::util::AsPartial>::Partial >);
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

        #partial_struct
    };

    // std::fs::write(format!("/tmp/blas__{}.rs", struct_name.to_string()), expanded.to_string()).unwrap();

    Ok(proc_macro::TokenStream::from(expanded))
}
