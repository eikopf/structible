use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Field, Ident, ItemStruct, Token, Type, Visibility};

use crate::util::extract_option_inner;

/// The backing map type specified in the attribute.
#[derive(Debug, Clone)]
pub enum BackingType {
    HashMap,
    BTreeMap,
}

impl BackingType {
    pub fn to_tokens(&self) -> TokenStream {
        match self {
            BackingType::HashMap => quote::quote! { ::std::collections::HashMap },
            BackingType::BTreeMap => quote::quote! { ::std::collections::BTreeMap },
        }
    }
}

/// Configuration parsed from `#[structible(...)]` attribute.
#[derive(Debug)]
pub struct StructibleConfig {
    pub backing: BackingType,
}

impl Parse for StructibleConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Default to HashMap if no arguments provided
        if input.is_empty() {
            return Ok(StructibleConfig {
                backing: BackingType::HashMap,
            });
        }

        // Try to parse as a shorthand (just the type name)
        let fork = input.fork();
        if let Ok(ident) = fork.parse::<Ident>() {
            // Check if this is followed by `=` (key-value) or nothing/comma (shorthand)
            if !fork.peek(Token![=]) {
                let backing = match ident.to_string().as_str() {
                    "HashMap" => BackingType::HashMap,
                    "BTreeMap" => BackingType::BTreeMap,
                    other => {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!(
                                "unknown backing type `{}`, expected `HashMap` or `BTreeMap`",
                                other
                            ),
                        ));
                    }
                };
                // Consume the identifier
                input.parse::<Ident>()?;
                return Ok(StructibleConfig { backing });
            }
        }

        // Parse as key-value pairs
        let mut backing = None;

        let pairs = Punctuated::<MetaItem, Token![,]>::parse_terminated(input)?;

        for item in pairs {
            match item.key.to_string().as_str() {
                "backing" => {
                    backing = Some(match item.value.to_string().as_str() {
                        "HashMap" => BackingType::HashMap,
                        "BTreeMap" => BackingType::BTreeMap,
                        other => {
                            return Err(syn::Error::new(
                                item.value.span(),
                                format!(
                                    "unknown backing type `{}`, expected `HashMap` or `BTreeMap`",
                                    other
                                ),
                            ));
                        }
                    });
                }
                other => {
                    return Err(syn::Error::new(
                        item.key.span(),
                        format!("unknown attribute `{}`", other),
                    ));
                }
            }
        }

        // Default to HashMap if backing was not specified
        let backing = backing.unwrap_or(BackingType::HashMap);

        Ok(StructibleConfig { backing })
    }
}

struct MetaItem {
    key: Ident,
    value: Ident,
}

impl Parse for MetaItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        let value: Ident = input.parse()?;
        Ok(MetaItem { key, value })
    }
}

/// Information about a single field in the struct.
pub struct FieldInfo {
    pub name: Ident,
    pub ty: Type,
    pub inner_ty: Type,
    pub is_optional: bool,
    pub vis: Visibility,
    pub attrs: Vec<Attribute>,
}

impl FieldInfo {
    pub fn from_field(field: &Field) -> syn::Result<Self> {
        let name = field.ident.clone().ok_or_else(|| {
            syn::Error::new_spanned(field, "structible only supports named fields")
        })?;

        let ty = field.ty.clone();
        let (is_optional, inner_ty) = match extract_option_inner(&ty) {
            Some(inner) => (true, inner.clone()),
            None => (false, ty.clone()),
        };

        Ok(FieldInfo {
            name,
            ty,
            inner_ty,
            is_optional,
            vis: field.vis.clone(),
            attrs: field.attrs.clone(),
        })
    }
}

/// Parse all fields from a struct.
pub fn parse_struct_fields(item: &ItemStruct) -> syn::Result<Vec<FieldInfo>> {
    let fields = match &item.fields {
        syn::Fields::Named(named) => &named.named,
        syn::Fields::Unnamed(_) => {
            return Err(syn::Error::new_spanned(
                item,
                "structible only supports structs with named fields",
            ));
        }
        syn::Fields::Unit => {
            return Err(syn::Error::new_spanned(
                item,
                "structible does not support unit structs",
            ));
        }
    };

    fields.iter().map(FieldInfo::from_field).collect()
}
