pub struct KeyEqualsLitStr{
    pub key: syn::Ident,
    pub equals_token: syn::Token![=],
    pub value: syn::LitStr,
}

impl syn::parse::Parse for KeyEqualsLitStr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self{
            key: input.parse()?,
            equals_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

