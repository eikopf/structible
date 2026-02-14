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

    let variants: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = to_pascal_case(&f.name);
            let attrs = &f.attrs;
            quote! {
                #(#attrs)*
                #variant
            }
        })
        .collect();

    quote! {
        #[doc(hidden)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum #enum_name {
            #(#variants),*
        }
    }
}

/// Generate the value enum (used as map values).
pub fn generate_value_enum(struct_name: &Ident, fields: &[FieldInfo]) -> TokenStream {
    let enum_name = value_enum_name(struct_name);

    let variants: Vec<_> = fields
        .iter()
        .map(|f| {
            let variant = to_pascal_case(&f.name);
            let ty = &f.inner_ty; // Always use inner type (unwrapped for Option)
            quote! { #variant(#ty) }
        })
        .collect();

    quote! {
        #[doc(hidden)]
        #[derive(Debug, Clone)]
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
    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);

    let constructor = generate_constructor(struct_name, fields, config);
    let getters = generate_getters(struct_name, fields);
    let getters_mut = generate_getters_mut(struct_name, fields);
    let setters = generate_setters(struct_name, fields);
    let removers = generate_removers(struct_name, fields);

    quote! {
        impl #struct_name {
            #constructor
            #(#getters)*
            #(#getters_mut)*
            #(#setters)*
            #(#removers)*

            /// Returns a reference to the value for the given field.
            pub fn get(&self, field: &#field_enum) -> Option<&#value_enum> {
                self.inner.get(field)
            }

            /// Inserts a value, returning the previous value if present.
            pub fn insert(&mut self, field: #field_enum, value: #value_enum) -> Option<#value_enum> {
                self.inner.insert(field, value)
            }

            /// Returns true if the field is present in the map.
            pub fn contains(&self, field: &#field_enum) -> bool {
                self.inner.contains_key(field)
            }

            /// Returns the number of fields present in the map.
            pub fn len(&self) -> usize {
                self.inner.len()
            }

            /// Returns true if no fields are present.
            pub fn is_empty(&self) -> bool {
                self.inner.is_empty()
            }

            /// Iterates over all present field-value pairs.
            pub fn iter(&self) -> impl Iterator<Item = (&#field_enum, &#value_enum)> {
                self.inner.iter()
            }
        }
    }
}

fn generate_constructor(
    struct_name: &Ident,
    fields: &[FieldInfo],
    config: &StructibleConfig,
) -> TokenStream {
    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);
    let map_type = config.backing.to_tokens();

    // Only required (non-optional) fields in constructor
    let required: Vec<_> = fields.iter().filter(|f| !f.is_optional).collect();

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

    quote! {
        /// Creates a new instance with all required fields.
        pub fn new(#(#params),*) -> Self {
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
        .map(|f| {
            let name = &f.name;
            let variant = to_pascal_case(name);

            let vis = &f.vis;

            if f.is_optional {
                let inner_ty = &f.inner_ty;
                quote! {
                    #vis fn #name(&self) -> Option<&#inner_ty> {
                        match self.inner.get(&#field_enum::#variant) {
                            Some(#value_enum::#variant(v)) => Some(v),
                            _ => None,
                        }
                    }
                }
            } else {
                let ty = &f.ty;
                quote! {
                    #vis fn #name(&self) -> &#ty {
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
        .map(|f| {
            let name = &f.name;
            let getter_mut_name = format_ident!("{}_mut", name);
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
        .map(|f| {
            let name = &f.name;
            let setter_name = format_ident!("set_{}", name);
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

fn generate_removers(struct_name: &Ident, fields: &[FieldInfo]) -> Vec<TokenStream> {
    let field_enum = field_enum_name(struct_name);
    let value_enum = value_enum_name(struct_name);

    // Only optional fields can be removed
    fields
        .iter()
        .filter(|f| f.is_optional)
        .map(|f| {
            let name = &f.name;
            let remover_name = format_ident!("remove_{}", name);
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
