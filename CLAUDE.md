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
  - `util.rs` - Helper functions: `extract_option_inner` for unwrapping `Option<T>`, `to_pascal_case` for enum variant names

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
1. `__StructibleField_Person` - Hidden enum for map keys (one variant per field)
2. `__StructibleValue_Person` - Hidden enum for map values (wraps each field type)
3. `PersonFields` - Companion struct for ownership extraction via `into_fields()`
4. `Person` struct with an `inner: HashMap<__StructibleField_Person, __StructibleValue_Person>` field
5. Generated methods:
   - Constructor (`new` or custom name via `constructor = name`)
   - Getters, mutable getters, setters for all fields
   - Removers for optional fields
   - `take_*` methods for extracting owned values (panics for required fields if missing)
   - `into_fields()` for full ownership extraction into the companion struct
   - `len()` and `is_empty()` for querying map size
6. `Default` impl (only if all fields are optional)

### Attribute Syntax

**Struct-level:**
- `#[structible(HashMap)]` - Shorthand for backing type (defaults to `HashMap`)
- `#[structible(backing = BTreeMap)]` - Explicit backing type
- `#[structible(backing = HashMap, constructor = create)]` - Custom constructor name

**Field-level:**
- `#[structible(get = custom_getter)]` - Custom getter name
- `#[structible(get_mut = custom_mut)]` - Custom mutable getter name
- `#[structible(set = custom_setter)]` - Custom setter name
- `#[structible(remove = custom_remover)]` - Custom remover name (optional fields only)
- `#[structible(key = KeyType)]` - Unknown/extension fields catch-all

### Key Design Decisions

- Optional fields (`Option<T>`) are stored without the `Option` wrapper; presence/absence in the map represents `Some`/`None`
- Required field getters panic if the field is missing (invariant violation)
- Unknown/extension fields use a `#[structible(key = Type)]` attribute and generate an `Unknown(K)` variant in the field enum
- The field enum derives `Copy` only when there's no unknown field (since unknown keys may not be `Copy`)
- Unknown fields require the `IterableMap` trait for iteration support
