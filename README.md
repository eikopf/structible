# `structible`

This crate provides the `structible` attribute macro for defining structs
backed by a map type (`HashMap` or `BTreeMap`) rather than native struct
storage. It generates type-safe field and value enums, along with accessor
methods, while preserving the external interface of a conventional struct.

```rust
use structible::structible;

#[structible(backing = HashMap)]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub email: Option<String>,
}

let mut person = Person::new("Alice".into(), 30);
assert_eq!(person.name(), "Alice");
assert_eq!(*person.age(), 30);
assert_eq!(person.email(), None);

person.set_email(Some("alice@example.com".into()));
assert_eq!(person.email(), Some(&"alice@example.com".into()));

*person.age_mut() += 1;
assert_eq!(*person.age(), 31);

let removed = person.remove_email();
assert_eq!(removed, Some("alice@example.com".into()));
assert_eq!(person.email(), None);
```

## Motivation

In certain contexts—such as dynamic configuration systems, schema-driven
data models, or interop with loosely-typed external systems—it can be
useful to back a struct with a map rather than native fields. This allows
for runtime introspection of field presence, dynamic field access patterns,
and easier serialization to map-like formats.

However, abandoning the struct interface entirely sacrifices type safety
and ergonomics. `structible` bridges this gap by generating a struct that
*looks and feels* like a normal Rust struct from the caller's perspective,
while internally storing data in a `HashMap` or `BTreeMap`.

## Generated Methods

### Constructor

The `new` method accepts all *required* (non-`Option`) fields:

```rust
impl Person {
    pub fn new(name: String, age: u32) -> Self;
}
```

Optional fields start absent from the map and can be set via setters.

### Getters

For required fields, getters return a reference directly and panic if
the field is unexpectedly absent (a violation of type invariants):

```rust
pub fn name(&self) -> &String;
pub fn age(&self) -> &u32;
```

For optional fields, getters return `Option<&T>`:

```rust
pub fn email(&self) -> Option<&String>;
```

### Mutable Getters

Mutable access follows the same pattern:

```rust
pub fn name_mut(&mut self) -> &mut String;
pub fn age_mut(&mut self) -> &mut u32;
pub fn email_mut(&mut self) -> Option<&mut String>;
```

### Setters

For required fields, setters take the value directly:

```rust
pub fn set_name(&mut self, value: String);
pub fn set_age(&mut self, value: u32);
```

For optional fields, setters take `Option<T>`. Passing `None` removes
the field from the map:

```rust
pub fn set_email(&mut self, value: Option<String>);
```

### Removers

Only optional fields have remover methods, which remove the field from
the map and return the value if present:

```rust
pub fn remove_email(&mut self) -> Option<String>;
```

### Utility Methods

Methods for querying the struct's state:

```rust
pub fn len(&self) -> usize;      // Number of fields present
pub fn is_empty(&self) -> bool;  // True if no fields are present
```

## Attributes

The macro accepts the following attributes:

### `backing`

**Optional.** Specifies the map type backing the struct. Defaults to `HashMap`.

| Value | Map Type |
|-------|----------|
| `HashMap` | `std::collections::HashMap` (default) |
| `BTreeMap` | `std::collections::BTreeMap` |

```rust
#[structible]                       // Uses HashMap by default
struct Foo { /* ... */ }

#[structible(BTreeMap)]             // Shorthand for BTreeMap
struct Bar { /* ... */ }

#[structible(backing = BTreeMap)]   // Explicit key-value form
struct Baz { /* ... */ }
```

Use `BTreeMap` when you need deterministic iteration order or when field
keys should be `Ord` rather than `Hash`.

## Design Decisions

### Option Unwrapping

Fields declared as `Option<T>` are stored internally without the `Option`
wrapper. The `Option` semantics are represented by the field's presence
or absence in the backing map:

- **Present in map** → `Some(value)`
- **Absent from map** → `None`

This design avoids double-wrapping (`Option<Option<T>>` ambiguity) and
aligns with map semantics where missing keys naturally represent absence.

### Panic on Missing Required Fields

Getters for required fields panic if the field is not present in the map.
This is intentional: a missing required field represents a violation of
the type's invariants, analogous to undefined behavior at the type level.

The `new` constructor guarantees all required fields are present, so this
panic should not occur during normal usage.

### No `Default` Implementation

The generated struct does not implement `Default` because required fields
have no sensible default values. Use the `new` constructor instead.

## Limitations

- Only named struct fields are supported (no tuple structs or unit structs)
- Field types must be `Clone` for the value enum to derive `Clone`
- Generic structs are not currently supported

## License

MIT
