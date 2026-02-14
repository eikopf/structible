use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Ident, Visibility};

use crate::parse::{FieldInfo, StructibleConfig};
use crate::util::to_pascal_case;

/// Returns the hidden field enum name for a struct.
pub fn field_enum_name(struct_name: &Ident) -> Ident {
    format_ident!("__StructibleField_{}", struct_name)
}

/// Returns the hidden value enum name for a struct.
pub fn value_enum_name(struct_name: &Ident) -> Ident {
    format_ident!("__StructibleValue_{}", struct_name)
}

/// Generate the field enum (used as map keys).
pub fn generate_field_enum(struct_name: &Ident, fields: &[FieldInfo]) -> TokenStream {
    let enum_name = field_enum_name(struct_name);

    // Find unknown field if present
    let unknown_field = fields.iter().find(|f| f.is_unknown_field());

    // Generate variants for known fields only
    let known_variants: Vec<_> = fields
        .iter()
        .filter(|f| !f.is_unknown_field())
        .map(|f| {
            let variant = to_pascal_case(&f.name);
            let attrs = &f.attrs;
            quote! {
                #(#attrs)*
                #variant
            }
        })
        .collect();

    if let Some(uf) = unknown_field {
        // Generate generic enum with Unknown variant
        let key_type = uf.unknown_key_type().unwrap();
        quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
            pub enum #enum_name<__K = #key_type> {
                #(#known_variants,)*
                Unknown(__K),
            }
        }
    } else {
        // No unknown field - generate simple enum with Copy
        quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
            pub enum #enum_name {
                #(#known_variants),*
            }
        }
    }
}

/// Generate the value enum (used as map values).
pub fn generate_value_enum(struct_name: &Ident, fields: &[FieldInfo]) -> TokenStream {
    let enum_name = value_enum_name(struct_name);

    // Find unknown field if present
    let unknown_field = fields.iter().find(|f| f.is_unknown_field());

    // Generate variants for known fields only
    let mut variants: Vec<_> = fields
        .iter()
        .filter(|f| !f.is_unknown_field())
        .map(|f| {
            let variant = to_pascal_case(&f.name);
            let ty = &f.inner_ty; // Always use inner type (unwrapped for Option)
            quote! { #variant(#ty) }
        })
        .collect();

    // Add Unknown variant if there's an unknown field
    if let Some(uf) = unknown_field {
        let value_ty = &uf.inner_ty;
        variants.push(quote! { Unknown(#value_ty) });
    }

    quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, PartialEq)]
        pub enum #enum_name {
            #(#variants),*
        }
    }
}

/// Generate the struct definition.
pub fn generate_struct(
    struct_name: &Ident,
    vis: &Visibility,
    config: &StructibleConfig,
    attrs: &[Attribute],
) -> TokenStream {
    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);
    let map_type = config.backing.to_tokens();

    quote! {
        #[derive(Debug, Clone, PartialEq)]
        #(#attrs)*
        #vis struct #struct_name {
            inner: #map_type<#field_enum, #value_enum>,
        }
    }
}

/// Generate the impl block with all methods.
pub fn generate_impl(
    struct_name: &Ident,
    fields: &[FieldInfo],
    config: &StructibleConfig,
) -> TokenStream {
    let constructor = generate_constructor(struct_name, fields, config);
    let getters = generate_getters(struct_name, fields);
    let getters_mut = generate_getters_mut(struct_name, fields);
    let setters = generate_setters(struct_name, fields);
    let removers = generate_removers(struct_name, fields);
    let unknown_methods = generate_unknown_field_methods(struct_name, fields);

    quote! {
        impl #struct_name {
            #constructor
            #(#getters)*
            #(#getters_mut)*
            #(#setters)*
            #(#removers)*
            #unknown_methods

            /// Returns the number of fields currently present.
            pub fn len(&self) -> usize {
                self.inner.len()
            }

            /// Returns true if no fields are present.
            pub fn is_empty(&self) -> bool {
                self.inner.is_empty()
            }
        }
    }
}

/// Generate a Default impl if all fields are optional.
pub fn generate_default_impl(
    struct_name: &Ident,
    fields: &[FieldInfo],
    config: &StructibleConfig,
) -> Option<TokenStream> {
    // Only generate Default if all non-unknown fields are optional
    // (Unknown fields are always optional by validation)
    let all_optional = fields
        .iter()
        .filter(|f| !f.is_unknown_field())
        .all(|f| f.is_optional);
    if !all_optional {
        return None;
    }

    let map_type = config.backing.to_tokens();

    Some(quote! {
        impl ::std::default::Default for #struct_name {
            fn default() -> Self {
                Self {
                    inner: #map_type::new(),
                }
            }
        }
    })
}

fn generate_constructor(
    struct_name: &Ident,
    fields: &[FieldInfo],
    config: &StructibleConfig,
) -> TokenStream {
    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);
    let map_type = config.backing.to_tokens();

    // Only required (non-optional) fields in constructor, excluding unknown fields
    let required: Vec<_> = fields
        .iter()
        .filter(|f| !f.is_optional && !f.is_unknown_field())
        .collect();

    let params: Vec<_> = required
        .iter()
        .map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            quote! { #name: #ty }
        })
        .collect();

    let inserts: Vec<_> = required
        .iter()
        .map(|f| {
            let name = &f.name;
            let variant = to_pascal_case(&f.name);
            quote! {
                inner.insert(#field_enum::#variant, #value_enum::#variant(#name));
            }
        })
        .collect();

    let constructor_name = config
        .constructor
        .clone()
        .unwrap_or_else(|| format_ident!("new"));

    quote! {
        /// Creates a new instance with all required fields.
        pub fn #constructor_name(#(#params),*) -> Self {
            let mut inner = #map_type::new();
            #(#inserts)*
            Self { inner }
        }
    }
}

fn generate_getters(struct_name: &Ident, fields: &[FieldInfo]) -> Vec<TokenStream> {
    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);

    fields
        .iter()
        .filter(|f| !f.is_unknown_field())
        .map(|f| {
            let name = &f.name;
            let getter_name = f.config.get.clone().unwrap_or_else(|| name.clone());
            let variant = to_pascal_case(name);

            let vis = &f.vis;

            if f.is_optional {
                let inner_ty = &f.inner_ty;
                quote! {
                    #vis fn #getter_name(&self) -> Option<&#inner_ty> {
                        match self.inner.get(&#field_enum::#variant) {
                            Some(#value_enum::#variant(v)) => Some(v),
                            _ => None,
                        }
                    }
                }
            } else {
                let ty = &f.ty;
                quote! {
                    #vis fn #getter_name(&self) -> &#ty {
                        match self.inner.get(&#field_enum::#variant) {
                            Some(#value_enum::#variant(v)) => v,
                            _ => panic!("required field `{}` not present", stringify!(#name)),
                        }
                    }
                }
            }
        })
        .collect()
}

fn generate_getters_mut(struct_name: &Ident, fields: &[FieldInfo]) -> Vec<TokenStream> {
    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);

    fields
        .iter()
        .filter(|f| !f.is_unknown_field())
        .map(|f| {
            let name = &f.name;
            let getter_mut_name = f
                .config
                .get_mut
                .clone()
                .unwrap_or_else(|| format_ident!("{}_mut", name));
            let variant = to_pascal_case(name);
            let vis = &f.vis;

            if f.is_optional {
                let inner_ty = &f.inner_ty;
                quote! {
                    #vis fn #getter_mut_name(&mut self) -> Option<&mut #inner_ty> {
                        match self.inner.get_mut(&#field_enum::#variant) {
                            Some(#value_enum::#variant(v)) => Some(v),
                            _ => None,
                        }
                    }
                }
            } else {
                let ty = &f.ty;
                quote! {
                    #vis fn #getter_mut_name(&mut self) -> &mut #ty {
                        match self.inner.get_mut(&#field_enum::#variant) {
                            Some(#value_enum::#variant(v)) => v,
                            _ => panic!("required field `{}` not present", stringify!(#name)),
                        }
                    }
                }
            }
        })
        .collect()
}

fn generate_setters(struct_name: &Ident, fields: &[FieldInfo]) -> Vec<TokenStream> {
    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);

    fields
        .iter()
        .filter(|f| !f.is_unknown_field())
        .map(|f| {
            let name = &f.name;
            let setter_name = f
                .config
                .set
                .clone()
                .unwrap_or_else(|| format_ident!("set_{}", name));
            let variant = to_pascal_case(name);
            let vis = &f.vis;

            if f.is_optional {
                let inner_ty = &f.inner_ty;
                quote! {
                    #vis fn #setter_name(&mut self, value: Option<#inner_ty>) {
                        match value {
                            Some(v) => {
                                self.inner.insert(#field_enum::#variant, #value_enum::#variant(v));
                            }
                            None => {
                                self.inner.remove(&#field_enum::#variant);
                            }
                        }
                    }
                }
            } else {
                let ty = &f.ty;
                quote! {
                    #vis fn #setter_name(&mut self, value: #ty) {
                        self.inner.insert(#field_enum::#variant, #value_enum::#variant(value));
                    }
                }
            }
        })
        .collect()
}

/// Generate methods for the unknown fields catch-all.
fn generate_unknown_field_methods(struct_name: &Ident, fields: &[FieldInfo]) -> TokenStream {
    let Some(unknown_field) = fields.iter().find(|f| f.is_unknown_field()) else {
        return quote! {};
    };

    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);
    let name = &unknown_field.name;
    let key_type = unknown_field.unknown_key_type().unwrap();
    let value_type = &unknown_field.inner_ty;
    let vis = &unknown_field.vis;

    // Method names derived from field name
    let add_method = format_ident!("add_{}", name);
    let get_method = name.clone();
    let get_mut_method = format_ident!("{}_mut", name);
    let remove_method = format_ident!("remove_{}", name);
    let iter_method = format_ident!("{}_iter", name);

    quote! {
        /// Inserts an unknown field with the given key and value.
        /// Returns the previous value if the key was already present.
        #vis fn #add_method(&mut self, key: #key_type, value: #value_type) -> Option<#value_type> {
            match self.inner.insert(
                #field_enum::Unknown(key),
                #value_enum::Unknown(value)
            ) {
                Some(#value_enum::Unknown(v)) => Some(v),
                _ => None,
            }
        }

        /// Returns a reference to the value for the given unknown key.
        #vis fn #get_method<__Q>(&self, key: &__Q) -> Option<&#value_type>
        where
            #key_type: ::std::borrow::Borrow<__Q>,
            __Q: ::std::hash::Hash + ::std::cmp::Eq + ?Sized,
        {
            // We need to iterate and find because HashMap::get requires the exact key type
            // For borrowed lookups, we compare via Borrow
            for (k, v) in self.inner.iter() {
                if let #field_enum::Unknown(ref stored_key) = k {
                    if <#key_type as ::std::borrow::Borrow<__Q>>::borrow(stored_key) == key {
                        if let #value_enum::Unknown(ref val) = v {
                            return Some(val);
                        }
                    }
                }
            }
            None
        }

        /// Returns a mutable reference to the value for the given unknown key.
        #vis fn #get_mut_method<__Q>(&mut self, key: &__Q) -> Option<&mut #value_type>
        where
            #key_type: ::std::borrow::Borrow<__Q>,
            __Q: ::std::hash::Hash + ::std::cmp::Eq + ?Sized,
        {
            for (k, v) in self.inner.iter_mut() {
                if let #field_enum::Unknown(ref stored_key) = k {
                    if <#key_type as ::std::borrow::Borrow<__Q>>::borrow(stored_key) == key {
                        if let #value_enum::Unknown(ref mut val) = v {
                            return Some(val);
                        }
                    }
                }
            }
            None
        }

        /// Removes an unknown field and returns the value if present.
        #vis fn #remove_method(&mut self, key: &#key_type) -> Option<#value_type> {
            match self.inner.remove(&#field_enum::Unknown(key.clone())) {
                Some(#value_enum::Unknown(v)) => Some(v),
                _ => None,
            }
        }

        /// Returns an iterator over all unknown fields.
        #vis fn #iter_method(&self) -> impl Iterator<Item = (&#key_type, &#value_type)> {
            self.inner.iter().filter_map(|(k, v)| {
                match (k, v) {
                    (#field_enum::Unknown(key), #value_enum::Unknown(value)) => Some((key, value)),
                    _ => None,
                }
            })
        }
    }
}

fn generate_removers(struct_name: &Ident, fields: &[FieldInfo]) -> Vec<TokenStream> {
    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);

    // Only optional fields can be removed, and skip unknown fields
    fields
        .iter()
        .filter(|f| f.is_optional && !f.is_unknown_field())
        .map(|f| {
            let name = &f.name;
            let remover_name = f
                .config
                .remove
                .clone()
                .unwrap_or_else(|| format_ident!("remove_{}", name));
            let variant = to_pascal_case(name);
            let inner_ty = &f.inner_ty;
            let vis = &f.vis;

            quote! {
                /// Removes the field and returns the value if it was present.
                #vis fn #remover_name(&mut self) -> Option<#inner_ty> {
                    match self.inner.remove(&#field_enum::#variant) {
                        Some(#value_enum::#variant(v)) => Some(v),
                        _ => None,
                    }
                }
            }
        })
        .collect()
}
