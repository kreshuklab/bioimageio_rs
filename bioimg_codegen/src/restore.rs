use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};

/// The `saved_data=path::to::SavedDataType` in
/// `#[restore(saved_data=path::to::SavedDataType)]` for setting the `SavedData`
/// associated type in the generated `impl Restore`
struct SavedDataTypeConfig{
    #[allow(dead_code)]
    name_key: syn::Ident,
    #[allow(dead_code)]
    equals_sign: syn::Token![=],
    saved_data_type_path: syn::Type,
}

impl syn::parse::Parse for SavedDataTypeConfig{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        match ident.to_string().as_str() {
            "saved_data" =>  Ok(SavedDataTypeConfig {
                name_key: ident,
                equals_sign: input.parse()?,
                saved_data_type_path: input.parse()?,
            }.into()),
            _ => Err(syn::Error::new(
                ident.span(),
                format!("Unrecognized Restore config. Expected 'saved_data', found '{ident}'")
            ))
        }
    }
}


/// The attributes applied to the type (not the fields!) that is having
/// `Restore` derived on
struct RestoreDeriveConfig {
    saved_data_type_conf: SavedDataTypeConfig,
}

impl RestoreDeriveConfig {
    fn try_from_attrs(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut out = Err(syn::Error::new(
            Span::call_site(),
            "No saved_data type configuration. Expected #[restore(saved_data=path::to::SavedDataType)]"
        ));
        for attr in attrs{
            if attr.path().segments.first().unwrap().ident.to_string() != "restore" {
                continue
            }
            let syn::Meta::List(meta_list) = &attr.meta else {
                return Err(syn::Error::new(attr.meta.span(), "Expected key = value"))
            };
            let saved_data_type = meta_list.parse_args::<SavedDataTypeConfig>()?;
            out = Ok(Self{saved_data_type_conf: saved_data_type});
        }
        out
    }
}


/// Determines how a field is to be restored when deriving the `Restore` trait
enum FieldRestoreMode {
    /// The usual behavior of restoring this field from the `SavedData` value.
    /// It's the strategy used when no `#[restore(...)]` attribute is applied to
    /// a field.
    FromSavedData,
    /// Restore this field to `Default::default()` instead of getting a value
    /// out of `Restore::SavedData`. Activated by annotating a field with
    /// `#[restore(default)]`
    CallDefault,
    /// Run `self.update` after restoring all fields to restore this field
    /// instead of getting a value out of `Restore::SavedData`. Activated by
    /// annotating a field with `#[restore(on_default)]`
    OnUpdate(syn::Ident),
}

impl syn::parse::Parse for FieldRestoreMode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        match ident.to_string().as_str() {
            "default" => Ok(FieldRestoreMode::CallDefault),
            "on_update" => Ok(FieldRestoreMode::OnUpdate(ident)),
            _ => Err(syn::Error::new(ident.span(), "Unexpected config, expected 'default' or 'on_update'"))
        }
    }
}

impl FieldRestoreMode {
    /// Parse fields attributes as configurations for the `Restore` derive.
    /// Looks for either `#[restore(default)` or `#[restore(on_update)]` (or
    /// neither) to determine the strategy for restoring the field. Check
    /// `FieldRestoreMode` variants for more information.
    pub fn try_from_attrs(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut mode: Option<FieldRestoreMode> = None;
        for attr in attrs {
            if attr.path().segments.last().unwrap().ident.to_string() != "restore" {
                continue
            }
            let syn::Meta::List(meta_list) = &attr.meta else {
                return Err(syn::Error::new(attr.span(), "Expected restore(default) or restore(on_update)"))
            };
            match meta_list.parse_args::<FieldRestoreMode>()? {
                new_mode @ FieldRestoreMode::CallDefault | new_mode @ FieldRestoreMode::OnUpdate(_)=> {
                    if let Some(_) = mode.replace(new_mode){
                        return Err(syn::Error::new(meta_list.span(), "Setting restore mode again"))
                    }
                },
                FieldRestoreMode::FromSavedData => unreachable!("Restoring from msg is not configured from attr"),
            }
        }
        Ok(mode.unwrap_or(FieldRestoreMode::FromSavedData))
    }
    pub fn skips_dump(&self) -> bool{
        !matches!(self, Self::FromSavedData)
    }
}

pub fn do_derive_restore(input: TokenStream) -> syn::Result<TokenStream>{
    let input = syn::parse::<syn::ItemStruct>(input)?;
    let struct_name = &input.ident;
    let RestoreDeriveConfig { saved_data_type_conf } = RestoreDeriveConfig::try_from_attrs(&input.attrs)?;
    let saved_data_type = saved_data_type_conf.saved_data_type_path;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut saved_data_field_initializers = Vec::<TokenStream2>::new();
    for (field_idx, field) in input.fields.iter().enumerate(){
        let ident = field.ident.as_ref().map(|id| quote!(#id)).unwrap_or(quote!(#field_idx));
        let ident_span = ident.span();
        if FieldRestoreMode::try_from_attrs(&field.attrs)?.skips_dump(){
            continue;
        }
        saved_data_field_initializers.push(quote_spanned! {ident_span=>
            // FIXME: could we not use this path into bioimg_gui?
            #ident: crate::widgets::Restore::dump(&self.#ident),
        });
    }

    let mut restore_statements = Vec::<TokenStream2>::new();
    let mut update_trigger: Option<syn::Ident> = None;
    for (field_idx, field) in input.fields.iter().enumerate(){
        let ident = field.ident.as_ref().map(|id| quote!(#id)).unwrap_or(quote!(#field_idx));
        let span = ident.span();
        let ty_span = field.ty.span();

        let statement = match FieldRestoreMode::try_from_attrs(&field.attrs)?{
            FieldRestoreMode::CallDefault => quote_spanned! {ty_span=>
                self.#ident = std::default::Default::default();
            },
            FieldRestoreMode::OnUpdate(update_marker) => {
                update_trigger = Some(update_marker);
                quote!{}
            },
            FieldRestoreMode::FromSavedData => quote_spanned! {span=>
                // FIXME: could we not use this path into bioimg_gui?
                crate::widgets::Restore::restore(&mut self.#ident, saved_data.#ident);
            }
        };
        restore_statements.push(statement);
    }

    if let Some(attr) = update_trigger{
        let span = attr.span(); 
        restore_statements.push(quote_spanned! {span=>
            self.update();
        })
    }

    let expanded = quote! {
        impl #impl_generics crate::widgets::Restore for #struct_name #ty_generics #where_clause {
            type    SavedData = #saved_data_type;
            fn dump(&self) -> Self::SavedData #ty_generics{
                #saved_data_type {
                    #(#saved_data_field_initializers)*
                }
            }
            fn restore(&mut self, saved_data: Self::SavedData){
                #(#restore_statements)*
            }
        }
    };

    Ok(proc_macro::TokenStream::from(expanded))
}
