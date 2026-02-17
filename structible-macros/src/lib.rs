//! Procedural macro implementation for `structible`.
//!
//! This crate provides the `#[structible]` attribute macro, which transforms
//! structs into map-backed types with generated accessors. Users should depend
//! on the main `structible` crate, which re-exports this macro.
//!
//! # Design
//!
//! The macro generates several items from a single struct definition:
//!
//! - A **field enum** used as map keys (one variant per field)
//! - A **value enum** used as map values (wrapping each field's type)
//! - A **fields struct** for ownership extraction via `into_fields()`
//! - The **main struct** backed by the chosen map type
//! - An **impl block** with accessors for all fields
//!
//! # Invariants
//!
//! Required fields (non-`Option`) are guaranteed to be present after
//! construction. The generated constructor enforces this by requiring values
//! for all required fields. Getters for required fields return references
//! directly, not `Option`, because the field is always present.
//!
//! # Optional Field Storage
//!
//! Fields typed as `Option<T>` are stored as `T` in the backing map, not as
//! `Option<T>`. Presence or absence in the map represents `Some` or `None`.
//! This means:
//!
//! - Getters return `Option<&T>` (present = `Some`, absent = `None`)
//! - Setters accept `Option<T>` (`Some` inserts, `None` removes)
//! - Removers extract the value if present

extern crate proc_macro;

mod codegen;
mod parse;
mod util;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, parse_macro_input};

use crate::codegen::{
    generate_debug_impl, generate_default_impl, generate_field_enum, generate_fields_debug_impl,
    generate_fields_impl, generate_fields_struct, generate_impl, generate_struct,
    generate_value_enum,
};
use crate::parse::{StructibleConfig, parse_struct_fields};

/// Transforms a struct into a map-backed type with generated accessors.
///
/// # Example
///
/// ```ignore
/// use structible::structible;
///
/// #[structible(HashMap)]
/// pub struct Person {
///     pub name: String,
///     pub age: u32,
///     pub email: Option<String>,
/// }
/// ```
///
/// This generates:
/// - A field enum for map keys (one variant per field)
/// - A value enum for map values (wrapping each field type)
/// - A `PersonFields` struct for ownership extraction
/// - The `Person` struct backed by `HashMap`
/// - Getters, setters, and removers for each field
///
/// # Optional Fields
///
/// Fields typed as `Option<T>` are stored without the `Option` wrapper.
/// Presence in the map represents `Some`, absence represents `None`:
///
/// - `email()` returns `Option<&String>`
/// - `set_email(Some(v))` inserts the value
/// - `set_email(None)` removes the value
/// - `remove_email()` extracts and returns the value if present
///
/// # Required Fields
///
/// Non-optional fields are guaranteed present after construction:
///
/// - `name()` returns `&String` (not `Option`)
/// - `set_name(v)` replaces the value
/// - Use `into_fields()` then `take_name()` to extract owned value
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
    let fields_impl = generate_fields_impl(name, &fields, &config, generics);
    let fields_debug_impl = generate_fields_debug_impl(name, &fields, generics);
    let struct_def = generate_struct(name, vis, &config, attrs, generics);
    let debug_impl = generate_debug_impl(name, &fields, generics);
    let impl_block = generate_impl(name, &fields, &config, generics);
    let default_impl = generate_default_impl(name, &fields, &config, generics);

    let expanded = quote! {
        #field_enum
        #value_enum
        #fields_struct
        #fields_impl
        #fields_debug_impl
        #struct_def
        #debug_impl
        #impl_block
        #default_impl
    };

    expanded.into()
}
