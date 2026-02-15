extern crate proc_macro;

mod codegen;
mod parse;
mod util;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

use crate::codegen::{
    generate_default_impl, generate_field_enum, generate_fields_struct, generate_impl,
    generate_struct, generate_value_enum,
};
use crate::parse::{parse_struct_fields, StructibleConfig};

/// Transforms a struct into a map-backed type with generated accessors.
///
/// # Example
///
/// ```ignore
/// use structible::structible;
///
/// #[structible(backing = HashMap)]
/// pub struct Person {
///     pub name: String,
///     pub age: u32,
///     pub email: Option<String>,
/// }
/// ```
///
/// This generates:
/// - `PersonField` enum for map keys
/// - `PersonValue` enum for map values
/// - `Person` struct backed by `HashMap<PersonField, PersonValue>`
/// - Getters, setters, builders, and removers for each field
#[proc_macro_attribute]
pub fn structible(attr: TokenStream, item: TokenStream) -> TokenStream {
    let config = match syn::parse::<StructibleConfig>(attr) {
        Ok(c) => c,
        Err(e) => return e.to_compile_error().into(),
    };

    let input = parse_macro_input!(item as ItemStruct);

    let fields = match parse_struct_fields(&input) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error().into(),
    };

    let name = &input.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let generics = &input.generics;

    let field_enum = generate_field_enum(name, &fields);
    let value_enum = generate_value_enum(name, &fields, generics);
    let fields_struct = generate_fields_struct(name, vis, &fields, &config, generics);
    let struct_def = generate_struct(name, vis, &config, attrs, generics);
    let impl_block = generate_impl(name, &fields, &config, generics);
    let default_impl = generate_default_impl(name, &fields, &config, generics);

    let expanded = quote! {
        #field_enum
        #value_enum
        #fields_struct
        #struct_def
        #impl_block
        #default_impl
    };

    expanded.into()
}
