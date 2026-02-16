use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Field, Ident, ItemStruct, Token, Type, Visibility};

use crate::util::extract_option_inner;

/// The backing map type specified in the attribute.
///
/// This can be any type that implements `BackingMap<K, V>`.
#[derive(Clone)]
pub struct BackingType {
    ty: Type,
}

impl BackingType {
    pub fn to_tokens(&self) -> TokenStream {
        let ty = &self.ty;
        quote::quote! { #ty }
    }

    /// Create a BackingType from a parsed Type.
    ///
    /// The type is used as-is without any expansion or transformation.
    pub fn from_type(ty: Type) -> Self {
        Self { ty }
    }
}

impl Default for BackingType {
    fn default() -> Self {
        // Default to HashMap
        Self {
            ty: syn::parse_quote! { ::std::collections::HashMap },
        }
    }
}

/// Configuration parsed from `#[structible(...)]` attribute on the struct.
pub struct StructibleConfig {
    pub backing: BackingType,
    pub constructor: Option<Ident>,
    /// If true, generate `len()` and `is_empty()` methods.
    pub with_len: bool,
}

/// Configuration parsed from `#[structible(...)]` attribute on a field.
#[derive(Default, Clone)]
pub struct FieldConfig {
    pub get: Option<Ident>,
    pub get_mut: Option<Ident>,
    pub set: Option<Ident>,
    pub remove: Option<Ident>,
    /// If present, this field is an unknown fields catch-all with the given key type.
    pub unknown_key: Option<Type>,
}

impl Parse for StructibleConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Default to HashMap if no arguments provided
        if input.is_empty() {
            return Ok(StructibleConfig {
                backing: BackingType::default(),
                constructor: None,
                with_len: false,
            });
        }

        // Try to parse as a shorthand (just a type, not key = value or flag)
        // We detect this by checking if it looks like `backing = ...` or `constructor = ...`
        // or a known flag like `with_len`
        let fork = input.fork();
        if let Ok(first_ident) = fork.parse::<Ident>() {
            let is_key_value = fork.peek(Token![=]);
            let is_flag = first_ident == "with_len";
            let has_more = fork.peek(Token![,]);
            if !is_key_value && !is_flag && !has_more {
                // This is a shorthand type specification
                // Parse the full type (could be `HashMap`, `indexmap::IndexMap`, etc.)
                let ty: Type = input.parse()?;
                let backing = BackingType::from_type(ty);
                return Ok(StructibleConfig {
                    backing,
                    constructor: None,
                    with_len: false,
                });
            }
        }

        // Parse as comma-separated items (key-value pairs or flags)
        let mut backing = None;
        let mut constructor = None;
        let mut with_len = false;

        while !input.is_empty() {
            let key: Ident = input.parse()?;

            match key.to_string().as_str() {
                "backing" => {
                    let _: Token![=] = input.parse()?;
                    let ty: Type = input.parse()?;
                    backing = Some(BackingType::from_type(ty));
                }
                "constructor" => {
                    let _: Token![=] = input.parse()?;
                    let ty: Type = input.parse()?;
                    // Constructor expects an identifier, not a type
                    let ident = match ty {
                        Type::Path(ref p) if p.path.get_ident().is_some() => {
                            p.path.get_ident().unwrap().clone()
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                ty,
                                "constructor must be an identifier",
                            ));
                        }
                    };
                    constructor = Some(ident);
                }
                "with_len" => {
                    with_len = true;
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown attribute `{}`", other),
                    ));
                }
            }

            // Consume optional comma
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }

        // Default to HashMap if backing was not specified
        let backing = backing.unwrap_or_default();

        Ok(StructibleConfig {
            backing,
            constructor,
            with_len,
        })
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
    pub config: FieldConfig,
}

impl FieldInfo {
    /// Returns true if this field is an unknown fields catch-all.
    pub fn is_unknown_field(&self) -> bool {
        self.config.unknown_key.is_some()
    }

    /// Returns the key type for unknown fields, if this is an unknown field.
    pub fn unknown_key_type(&self) -> Option<&Type> {
        self.config.unknown_key.as_ref()
    }

    pub fn from_field(field: &Field) -> syn::Result<Self> {
        let name = field.ident.clone().ok_or_else(|| {
            syn::Error::new_spanned(field, "structible only supports named fields")
        })?;

        let ty = field.ty.clone();
        let (is_optional, inner_ty) = match extract_option_inner(&ty) {
            Some(inner) => (true, inner.clone()),
            None => (false, ty.clone()),
        };

        // Parse field-level structible attributes
        let config = parse_field_config(&field.attrs)?;

        // Filter out structible attributes from the preserved attrs
        let attrs: Vec<_> = field
            .attrs
            .iter()
            .filter(|a| !a.path().is_ident("structible"))
            .cloned()
            .collect();

        Ok(FieldInfo {
            name,
            ty,
            inner_ty,
            is_optional,
            vis: field.vis.clone(),
            attrs,
            config,
        })
    }
}

/// Parse field-level `#[structible(...)]` attributes.
fn parse_field_config(attrs: &[Attribute]) -> syn::Result<FieldConfig> {
    let mut config = FieldConfig::default();

    for attr in attrs {
        if attr.path().is_ident("structible") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("get") {
                    let _: Token![=] = meta.input.parse()?;
                    let value: Ident = meta.input.parse()?;
                    config.get = Some(value);
                } else if meta.path.is_ident("get_mut") {
                    let _: Token![=] = meta.input.parse()?;
                    let value: Ident = meta.input.parse()?;
                    config.get_mut = Some(value);
                } else if meta.path.is_ident("set") {
                    let _: Token![=] = meta.input.parse()?;
                    let value: Ident = meta.input.parse()?;
                    config.set = Some(value);
                } else if meta.path.is_ident("remove") {
                    let _: Token![=] = meta.input.parse()?;
                    let value: Ident = meta.input.parse()?;
                    config.remove = Some(value);
                } else if meta.path.is_ident("key") {
                    let _: Token![=] = meta.input.parse()?;
                    let key_type: Type = meta.input.parse()?;
                    config.unknown_key = Some(key_type);
                } else {
                    return Err(meta.error(format!(
                        "unknown field attribute `{}`",
                        meta.path.get_ident().map_or("".into(), |i| i.to_string())
                    )));
                }
                Ok(())
            })?;
        }
    }

    Ok(config)
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

    let parsed: Vec<FieldInfo> = fields
        .iter()
        .map(FieldInfo::from_field)
        .collect::<Result<_, _>>()?;

    // Validate: at most one unknown field
    let unknown_fields: Vec<_> = parsed.iter().filter(|f| f.is_unknown_field()).collect();
    if unknown_fields.len() > 1 {
        return Err(syn::Error::new_spanned(
            item,
            "structible only supports one unknown fields catch-all per struct",
        ));
    }

    // Validate: unknown field must be Optional
    for field in &unknown_fields {
        if !field.is_optional {
            return Err(syn::Error::new_spanned(
                &field.name,
                "unknown fields catch-all must be declared as Option<T>",
            ));
        }
    }

    Ok(parsed)
}
