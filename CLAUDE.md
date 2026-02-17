# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

```bash
# Build all crates
cargo build

# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p structible
cargo test -p structible-macros

# Run a single test by name
cargo test test_basic_fields

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Architecture

This is a Rust workspace containing a proc-macro crate for generating map-backed structs.

### Crate Structure

- **`structible`** - Main crate that users depend on. Re-exports the `#[structible]` macro and defines the `BackingMap` and `IterableMap` traits with implementations for `HashMap` and `BTreeMap`.

- **`structible-macros`** - Proc-macro crate that implements the `#[structible]` attribute macro. Contains:
  - `lib.rs` - Entry point; orchestrates parsing and code generation
  - `parse.rs` - Parses struct and field attributes into `StructibleConfig`, `FieldConfig`, and `FieldInfo`
  - `codegen.rs` - Generates the field enum, value enum, fields struct, struct definition, impl block, and Default impl
  - `util.rs` - Helper functions: `extract_option_inner` for unwrapping `Option<T>`, `to_pascal_case` for enum variant names (handles raw identifiers like `r#type`)

### Code Generation

The macro transforms a struct like:
```rust
#[structible(HashMap)]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub email: Option<String>,
}
```

Into:
1. `__StructibleField_Person` - Hidden enum for map keys (one variant per known field)
2. `__StructibleValue_Person` - Hidden enum for map values (wraps each field's inner type)
3. `PersonFields` - Companion struct for ownership extraction via `into_fields()`
4. `Person` struct with an `inner: HashMap<__StructibleField_Person, __StructibleValue_Person>` field
5. Generated methods on main struct:
   - Constructor (`new` or custom name via `constructor = name`) - takes required fields only
   - Getters: `<field>()` - returns `&T` for required, `Option<&T>` for optional
   - Mutable getters: `<field>_mut()` - returns `&mut T` for required, `Option<&mut T>` for optional
   - Setters: `set_<field>(value)` - takes `T` (inner type for optional fields)
   - Removers: `remove_<field>()` - optional fields only, returns `Option<T>`
   - `into_fields()` - consumes struct, returns companion struct for extracting all fields
   - `len()` and `is_empty()` (opt-in via `with_len`)
6. Generated methods on `PersonFields` companion struct:
   - `take_<field>()` for ALL fields (required and optional), all return `Option<T>`
7. Derived traits: both structs derive `Clone, PartialEq` by default (opt-out via `no_clone`, `no_partial_eq`) with custom `Debug` impls (showing only present fields)
8. `Default` impl (only if all non-unknown fields are optional)

### Attribute Syntax

**Struct-level:**
- `#[structible(HashMap)]` - Shorthand for backing type (defaults to `HashMap`)
- `#[structible(backing = BTreeMap)]` - Explicit backing type
- `#[structible(backing = HashMap, constructor = create)]` - Custom constructor name
- `#[structible(with_len)]` - Enable `len()` and `is_empty()` methods
- `#[structible(no_clone)]` - Do not derive `Clone` on generated types (allows non-Clone field types like `&mut T`)
- `#[structible(no_partial_eq)]` - Do not derive `PartialEq` on generated types (allows non-PartialEq field types like `Box<dyn Fn()>`)

**Field-level:**
- `#[structible(get = custom_getter)]` - Custom getter name (replaces default `<field>`)
- `#[structible(get_mut = custom_mut)]` - Custom mutable getter name (replaces default `<field>_mut`)
- `#[structible(set = custom_setter)]` - Custom setter name (replaces default `set_<field>`)
- `#[structible(remove = custom_remover)]` - Custom remover name (optional fields only)
- `#[structible(key = KeyType)]` - Unknown/extension fields catch-all

### Unknown/Extension Fields

When a field has `#[structible(key = KeyType)]`, it becomes a catch-all for unknown keys:
- The field must be `Option<T>` (validated at compile time)
- At most one unknown field per struct

**Generated methods on main struct:**
- `insert_<field>(key, value)` - Insert unknown field, returns previous value if present
- `<field>(&key)` - Get by borrowed key (supports `Borrow` trait)
- `<field>_mut(&key)` - Mutable access by borrowed key
- `remove_<field>(&key)` - Remove and return value
- `<field>_iter()` - Iterate over all unknown fields as `(&K, &V)` pairs
- `<field>_iter_mut()` - Mutably iterate over all unknown fields as `(&K, &mut V)` pairs

**Generated methods on Fields companion struct:**
- `take_<field>(&key)` - Extract value for a specific unknown key
- `<field>_iter()` - Iterate unknown fields
- `<field>_iter_mut()` - Mutably iterate unknown fields
- `drain_<field>()` - Drain all unknown fields into a new map

### Key Design Decisions

- Optional fields (`Option<T>`) are stored without the `Option` wrapper; presence/absence in the map represents `Some`/`None`
- Required field getters/mutable getters panic if the field is missing (invariant violation)
- Setters for both required and optional fields take the value directly (`T`); use `remove_*` to clear optional fields
- The `Fields` companion struct has `take_*` for ALL fields, returning `Option<T>` (required fields should always be `Some` if struct was valid); use `into_fields()` to extract ownership
- `len()` and `is_empty()` are opt-in via `#[structible(with_len)]` to avoid conflicts with user-defined methods
- The field enum derives `Copy` only when there's no unknown field (unknown keys may not be `Copy`)
- Unknown fields require the `IterableMap` trait for iteration support
- Generics and lifetimes are fully supported; the value enum is parameterized with struct generics

### Traits

**`BackingMap<K, V>`** - Required for all backing types:
- `new()`, `with_capacity(usize)` (has default impl), `insert`, `get`, `get_mut`, `remove`, `len`, `is_empty`
- HashMap requires: `K: Eq + Hash`
- BTreeMap requires: `K: Ord`

**`IterableMap<K, V>`** - Required only when using unknown fields:
- `iter()` and `iter_mut()` for iterating over map entries
