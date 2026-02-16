use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Generics, Ident, Visibility};

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

/// Returns the companion fields struct name for ownership extraction.
pub fn fields_struct_name(struct_name: &Ident) -> Ident {
    format_ident!("{}Fields", struct_name)
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
            #[allow(non_camel_case_types, clippy::enum_variant_names)]
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
            #[allow(non_camel_case_types, clippy::enum_variant_names)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
            pub enum #enum_name {
                #(#known_variants),*
            }
        }
    }
}

/// Generate the value enum (used as map values).
pub fn generate_value_enum(
    struct_name: &Ident,
    fields: &[FieldInfo],
    generics: &Generics,
) -> TokenStream {
    let enum_name = value_enum_name(struct_name);
    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();

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
        #[allow(non_camel_case_types, clippy::enum_variant_names)]
        #[derive(Debug, Clone, PartialEq)]
        pub enum #enum_name #impl_generics #where_clause {
            #(#variants),*
        }
    }
}

/// Generate the companion fields struct for ownership extraction.
///
/// This struct mirrors the original but with real fields that can be destructured.
/// Optional fields are wrapped in `Option<T>`.
/// Unknown fields are collected into a map with the same backing type.
pub fn generate_fields_struct(
    struct_name: &Ident,
    vis: &Visibility,
    fields: &[FieldInfo],
    config: &StructibleConfig,
    generics: &Generics,
) -> TokenStream {
    let fields_struct = fields_struct_name(struct_name);
    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();
    let map_type = config.backing.to_tokens();

    // Generate struct fields (exclude unknown fields - they can't be statically typed)
    let struct_fields: Vec<_> = fields
        .iter()
        .filter(|f| !f.is_unknown_field())
        .map(|f| {
            let name = &f.name;
            let field_vis = &f.vis;
            let attrs = &f.attrs;

            if f.is_optional {
                // Optional fields get wrapped in Option
                let inner_ty = &f.inner_ty;
                quote! {
                    #(#attrs)*
                    #field_vis #name: Option<#inner_ty>
                }
            } else {
                // Required fields keep their type directly
                let ty = &f.ty;
                quote! {
                    #(#attrs)*
                    #field_vis #name: #ty
                }
            }
        })
        .collect();

    // Collect type parameters to ensure they're all used (some may only appear in unknown fields)
    // We add a PhantomData field if there are any type parameters
    let type_params: Vec<_> = generics.type_params().map(|tp| &tp.ident).collect();

    let phantom_field = if type_params.is_empty() {
        quote! {}
    } else {
        quote! {
            #[doc(hidden)]
            pub _phantom: ::std::marker::PhantomData<(#(#type_params,)*)>,
        }
    };

    // Generate unknown field if present
    let unknown_field = fields.iter().find(|f| f.is_unknown_field());
    let unknown_struct_field = if let Some(uf) = unknown_field {
        let name = &uf.name;
        let field_vis = &uf.vis;
        let key_type = uf.unknown_key_type().unwrap();
        let value_type = &uf.inner_ty;
        quote! {
            /// Unknown/extension fields collected from the original struct.
            #field_vis #name: #map_type<#key_type, #value_type>,
        }
    } else {
        quote! {}
    };

    quote! {
        /// Companion struct containing owned values of all fields.
        ///
        /// This struct can be destructured using pattern matching.
        #[derive(Debug, Clone, PartialEq)]
        #vis struct #fields_struct #impl_generics #where_clause {
            #(#struct_fields,)*
            #unknown_struct_field
            #phantom_field
        }
    }
}

/// Generate the struct definition.
pub fn generate_struct(
    struct_name: &Ident,
    vis: &Visibility,
    config: &StructibleConfig,
    attrs: &[Attribute],
    generics: &Generics,
) -> TokenStream {
    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);
    let map_type = config.backing.to_tokens();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[derive(Debug, Clone, PartialEq)]
        #(#attrs)*
        #vis struct #struct_name #impl_generics #where_clause {
            inner: #map_type<#field_enum, #value_enum #ty_generics>,
        }
    }
}

/// Generate the impl block with all methods.
pub fn generate_impl(
    struct_name: &Ident,
    fields: &[FieldInfo],
    config: &StructibleConfig,
    generics: &Generics,
) -> TokenStream {
    let constructor = generate_constructor(struct_name, fields, config, generics);
    let getters = generate_getters(struct_name, fields, generics);
    let getters_mut = generate_getters_mut(struct_name, fields, generics);
    let setters = generate_setters(struct_name, fields, generics);
    let removers = generate_removers(struct_name, fields, generics);
    let take_methods = generate_take_methods(struct_name, fields, generics);
    let into_fields = generate_into_fields(struct_name, fields, config, generics);
    let unknown_methods = generate_unknown_field_methods(struct_name, fields, generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let len_methods = if config.with_len {
        quote! {
            /// Returns the number of fields currently present.
            pub fn len(&self) -> usize {
                ::structible::BackingMap::len(&self.inner)
            }

            /// Returns true if no fields are present.
            pub fn is_empty(&self) -> bool {
                ::structible::BackingMap::is_empty(&self.inner)
            }
        }
    } else {
        quote! {}
    };

    quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #constructor
            #(#getters)*
            #(#getters_mut)*
            #(#setters)*
            #(#removers)*
            #(#take_methods)*
            #into_fields
            #unknown_methods
            #len_methods
        }
    }
}

/// Generate a Default impl if all fields are optional.
pub fn generate_default_impl(
    struct_name: &Ident,
    fields: &[FieldInfo],
    config: &StructibleConfig,
    generics: &Generics,
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

    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);
    let map_type = config.backing.to_tokens();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Some(quote! {
        impl #impl_generics ::std::default::Default for #struct_name #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    inner: <#map_type<#field_enum, #value_enum #ty_generics> as ::structible::BackingMap<#field_enum, #value_enum #ty_generics>>::new(),
                }
            }
        }
    })
}

fn generate_constructor(
    struct_name: &Ident,
    fields: &[FieldInfo],
    config: &StructibleConfig,
    generics: &Generics,
) -> TokenStream {
    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);
    let map_type = config.backing.to_tokens();
    let (_, ty_generics, _) = generics.split_for_impl();

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
                ::structible::BackingMap::insert(&mut inner, #field_enum::#variant, #value_enum::#variant(#name));
            }
        })
        .collect();

    let constructor_name = config
        .constructor
        .clone()
        .unwrap_or_else(|| format_ident!("new"));

    let required_count = required.len();

    quote! {
        /// Creates a new instance with all required fields.
        pub fn #constructor_name(#(#params),*) -> Self {
            let mut inner = <#map_type<#field_enum, #value_enum #ty_generics> as ::structible::BackingMap<#field_enum, #value_enum #ty_generics>>::with_capacity(#required_count);
            #(#inserts)*
            Self { inner }
        }
    }
}

fn generate_getters(
    struct_name: &Ident,
    fields: &[FieldInfo],
    _generics: &Generics,
) -> Vec<TokenStream> {
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
                        match ::structible::BackingMap::get(&self.inner, &#field_enum::#variant) {
                            Some(#value_enum::#variant(v)) => Some(v),
                            _ => None,
                        }
                    }
                }
            } else {
                let ty = &f.ty;
                quote! {
                    #vis fn #getter_name(&self) -> &#ty {
                        match ::structible::BackingMap::get(&self.inner, &#field_enum::#variant) {
                            Some(#value_enum::#variant(v)) => v,
                            _ => panic!("required field `{}` not present", stringify!(#name)),
                        }
                    }
                }
            }
        })
        .collect()
}

fn generate_getters_mut(
    struct_name: &Ident,
    fields: &[FieldInfo],
    _generics: &Generics,
) -> Vec<TokenStream> {
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
                        match ::structible::BackingMap::get_mut(&mut self.inner, &#field_enum::#variant) {
                            Some(#value_enum::#variant(v)) => Some(v),
                            _ => None,
                        }
                    }
                }
            } else {
                let ty = &f.ty;
                quote! {
                    #vis fn #getter_mut_name(&mut self) -> &mut #ty {
                        match ::structible::BackingMap::get_mut(&mut self.inner, &#field_enum::#variant) {
                            Some(#value_enum::#variant(v)) => v,
                            _ => panic!("required field `{}` not present", stringify!(#name)),
                        }
                    }
                }
            }
        })
        .collect()
}

fn generate_setters(
    struct_name: &Ident,
    fields: &[FieldInfo],
    _generics: &Generics,
) -> Vec<TokenStream> {
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
                                ::structible::BackingMap::insert(&mut self.inner, #field_enum::#variant, #value_enum::#variant(v));
                            }
                            None => {
                                ::structible::BackingMap::remove(&mut self.inner, &#field_enum::#variant);
                            }
                        }
                    }
                }
            } else {
                let ty = &f.ty;
                quote! {
                    #vis fn #setter_name(&mut self, value: #ty) {
                        ::structible::BackingMap::insert(&mut self.inner, #field_enum::#variant, #value_enum::#variant(value));
                    }
                }
            }
        })
        .collect()
}

/// Generate methods for the unknown fields catch-all.
fn generate_unknown_field_methods(
    struct_name: &Ident,
    fields: &[FieldInfo],
    _generics: &Generics,
) -> TokenStream {
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
            match ::structible::BackingMap::insert(
                &mut self.inner,
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
            // We need to iterate and find because the map's get requires the exact key type
            // For borrowed lookups, we compare via Borrow
            for (k, v) in ::structible::IterableMap::iter(&self.inner) {
                if let #field_enum::Unknown(stored_key) = k {
                    if <#key_type as ::std::borrow::Borrow<__Q>>::borrow(stored_key) == key {
                        if let #value_enum::Unknown(val) = v {
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
            for (k, v) in ::structible::IterableMap::iter_mut(&mut self.inner) {
                if let #field_enum::Unknown(stored_key) = k {
                    if <#key_type as ::std::borrow::Borrow<__Q>>::borrow(stored_key) == key {
                        if let #value_enum::Unknown(val) = v {
                            return Some(val);
                        }
                    }
                }
            }
            None
        }

        /// Removes an unknown field and returns the value if present.
        #vis fn #remove_method<__Q>(&mut self, key: &__Q) -> Option<#value_type>
        where
            #key_type: ::std::borrow::Borrow<__Q>,
            __Q: ::std::borrow::ToOwned<Owned = #key_type> + ::std::hash::Hash + ::std::cmp::Eq + ?Sized,
        {
            let owned_key: #key_type = key.to_owned();
            match ::structible::BackingMap::remove(&mut self.inner, &#field_enum::Unknown(owned_key)) {
                Some(#value_enum::Unknown(v)) => Some(v),
                _ => None,
            }
        }

        /// Returns an iterator over all unknown fields.
        #vis fn #iter_method(&self) -> impl Iterator<Item = (&#key_type, &#value_type)> {
            ::structible::IterableMap::iter(&self.inner).filter_map(|(k, v)| {
                match (k, v) {
                    (#field_enum::Unknown(key), #value_enum::Unknown(value)) => Some((key, value)),
                    _ => None,
                }
            })
        }
    }
}

fn generate_removers(
    struct_name: &Ident,
    fields: &[FieldInfo],
    _generics: &Generics,
) -> Vec<TokenStream> {
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
                    match ::structible::BackingMap::remove(&mut self.inner, &#field_enum::#variant) {
                        Some(#value_enum::#variant(v)) => Some(v),
                        _ => None,
                    }
                }
            }
        })
        .collect()
}

/// Generate `take_*` methods for extracting owned values from optional fields.
///
/// Only generated for optional fields to prevent leaving the struct in an invalid state.
/// Use `into_fields()` to extract ownership of required fields.
fn generate_take_methods(
    struct_name: &Ident,
    fields: &[FieldInfo],
    _generics: &Generics,
) -> Vec<TokenStream> {
    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);

    fields
        .iter()
        .filter(|f| f.is_optional && !f.is_unknown_field())
        .map(|f| {
            let name = &f.name;
            let take_name = format_ident!("take_{}", name);
            let variant = to_pascal_case(name);
            let vis = &f.vis;
            let inner_ty = &f.inner_ty;

            quote! {
                /// Removes and returns the field value if present.
                #vis fn #take_name(&mut self) -> Option<#inner_ty> {
                    match ::structible::BackingMap::remove(&mut self.inner, &#field_enum::#variant) {
                        Some(#value_enum::#variant(v)) => Some(v),
                        _ => None,
                    }
                }
            }
        })
        .collect()
}

/// Generate the `into_fields` method for full ownership extraction.
///
/// This method consumes the struct and returns a companion struct with all field values.
fn generate_into_fields(
    struct_name: &Ident,
    fields: &[FieldInfo],
    config: &StructibleConfig,
    generics: &Generics,
) -> TokenStream {
    let fields_struct = fields_struct_name(struct_name);
    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);
    let map_type = config.backing.to_tokens();
    let (_, ty_generics, _) = generics.split_for_impl();

    // Generate extraction for each known field
    let extractions: Vec<_> = fields
        .iter()
        .filter(|f| !f.is_unknown_field())
        .map(|f| {
            let name = &f.name;
            let variant = to_pascal_case(name);

            if f.is_optional {
                quote! {
                    let #name = match ::structible::BackingMap::remove(&mut inner, &#field_enum::#variant) {
                        Some(#value_enum::#variant(v)) => Some(v),
                        _ => None,
                    };
                }
            } else {
                quote! {
                    let #name = match ::structible::BackingMap::remove(&mut inner, &#field_enum::#variant) {
                        Some(#value_enum::#variant(v)) => v,
                        _ => panic!("required field `{}` not present", stringify!(#name)),
                    };
                }
            }
        })
        .collect();

    // Generate field names for struct construction
    let field_names: Vec<_> = fields
        .iter()
        .filter(|f| !f.is_unknown_field())
        .map(|f| &f.name)
        .collect();

    // Add phantom field initialization if there are type parameters
    let has_type_params = generics.type_params().next().is_some();
    let phantom_init = if has_type_params {
        quote! { _phantom: ::std::marker::PhantomData, }
    } else {
        quote! {}
    };

    // Generate unknown field collection if present
    let unknown_field = fields.iter().find(|f| f.is_unknown_field());
    let (unknown_extraction, unknown_init) = if let Some(uf) = unknown_field {
        let name = &uf.name;
        let key_type = uf.unknown_key_type().unwrap();
        let value_type = &uf.inner_ty;
        let extraction = quote! {
            // Collect unknown keys first, then remove them to build the output map
            let unknown_keys: ::std::vec::Vec<#key_type> = ::structible::IterableMap::iter(&inner)
                .filter_map(|(k, _)| {
                    if let #field_enum::Unknown(key) = k {
                        Some(key.clone())
                    } else {
                        None
                    }
                })
                .collect();
            let mut #name = <#map_type<#key_type, #value_type> as ::structible::BackingMap<#key_type, #value_type>>::new();
            for key in unknown_keys {
                if let Some(#value_enum::Unknown(value)) = ::structible::BackingMap::remove(&mut inner, &#field_enum::Unknown(key.clone())) {
                    ::structible::BackingMap::insert(&mut #name, key, value);
                }
            }
        };
        let init = quote! { #name, };
        (extraction, init)
    } else {
        (quote! {}, quote! {})
    };

    // Update doc comment based on whether unknown fields are collected
    let unknown_doc = if unknown_field.is_some() {
        quote! {
            /// Unknown/extension fields are collected into the returned struct.
        }
    } else {
        quote! {}
    };

    quote! {
        /// Consumes this struct and returns a companion struct with all owned field values.
        ///
        /// The returned struct can be destructured using pattern matching:
        /// ```ignore
        /// let Fields { name, age, .. } = value.into_fields();
        /// ```
        #unknown_doc
        ///
        /// # Panics
        /// Panics if any required field is missing (invariant violation).
        pub fn into_fields(self) -> #fields_struct #ty_generics {
            let mut inner = self.inner;
            #(#extractions)*
            #unknown_extraction
            #fields_struct {
                #(#field_names,)*
                #unknown_init
                #phantom_init
            }
        }
    }
}
