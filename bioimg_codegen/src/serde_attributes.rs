use crate::{syn_extensions::IAttrExt, util::KeyEqualsLitStr};


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

pub enum SerdeEnumTagParams{
    Untagged,
    InternallyTagged{tag_key: String},
    AdjacentlyTagged{tag_key: String, content_key: String},
}

impl SerdeEnumTagParams {
    pub fn try_from_attr(attr: &syn::Attribute) -> Option<Self>{
        if !attr.is_serde_attr(){
            return None
        }
        let syn::Meta::List(meta_list) = &attr.meta else {
            return None
        };
        meta_list.parse_args::<Self>().ok()
    }
}

impl syn::parse::Parse for SerdeEnumTagParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(ident) = input.fork().parse::<syn::Ident>(){
            if ident.to_string().as_str() == "untagged" {
                return Ok(Self::Untagged)
            }
        }

        let key_val1: KeyEqualsLitStr = input.parse()?;
        if input.is_empty(){
            if key_val1.key.to_string() != "tag" {
                return Err(syn::Error::new(key_val1.key.span(), "Expecting 'tag' key"))
            }
            return Ok(Self::InternallyTagged { tag_key: key_val1.value.value() })
        }
        let key_val2: KeyEqualsLitStr = input.parse()?;

        let (content_keyval, tag_keyval) = if key_val1.key.to_string() < key_val2.key.to_string() {
            (key_val1, key_val2)
        }else{
            (key_val2, key_val1)
        };

        if content_keyval.key.to_string() != "content"{
            return Err(syn::Error::new(content_keyval.key.span(), "expected key to be 'content'"))
        }
        if tag_keyval.key.to_string() != "tag"{
            return Err(syn::Error::new(tag_keyval.key.span(), "expected key to be 'tag'"))
        }

        Ok(Self::AdjacentlyTagged { tag_key: tag_keyval.value.value(), content_key: content_keyval.value.value() })
    }
}
