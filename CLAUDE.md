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

- **`structible`** - Main crate that users depend on. Re-exports the `#[structible]` macro and defines the `BackingMap` trait with implementations for `HashMap` and `BTreeMap`.

- **`structible-macros`** - Proc-macro crate that implements the `#[structible]` attribute macro. Contains:
  - `lib.rs` - Entry point; orchestrates parsing and code generation
  - `parse.rs` - Parses struct and field attributes into `StructibleConfig` and `FieldInfo`
  - `codegen.rs` - Generates the field enum, value enum, struct definition, and impl block
  - `util.rs` - Helper functions: `extract_option_inner` for unwrapping `Option<T>`, `to_pascal_case` for enum variant names

### Code Generation

The macro transforms a struct like:
```rust
#[structible(backing = HashMap)]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub email: Option<String>,
}
```

Into:
1. `__StructibleField_Person` - Hidden enum for map keys (one variant per field)
2. `__StructibleValue_Person` - Hidden enum for map values (wraps each field type)
3. `Person` struct with an `inner: HashMap<__StructibleField_Person, __StructibleValue_Person>` field
4. Generated methods: constructor, getters, mutable getters, setters, removers (for optional fields)

### Key Design Decisions

- Optional fields (`Option<T>`) are stored without the `Option` wrapper; presence/absence in the map represents `Some`/`None`
- Required field getters panic if the field is missing (invariant violation)
- Unknown/extension fields use a `#[structible(key = Type)]` attribute and generate an `Unknown(K)` variant in the field enum
- The field enum derives `Copy` only when there's no unknown field (since unknown keys may not be `Copy`)
