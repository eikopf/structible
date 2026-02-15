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

```rust,ignore
impl Person {
    pub fn new(name: String, age: u32) -> Self;
}
```

Optional fields start absent from the map and can be set via setters.

### Getters

For required fields, getters return a reference directly and panic if
the field is unexpectedly absent (a violation of type invariants):

```rust,ignore
pub fn name(&self) -> &String;
pub fn age(&self) -> &u32;
```

For optional fields, getters return `Option<&T>`:

```rust,ignore
pub fn email(&self) -> Option<&String>;
```

### Mutable Getters

Mutable access follows the same pattern:

```rust,ignore
pub fn name_mut(&mut self) -> &mut String;
pub fn age_mut(&mut self) -> &mut u32;
pub fn email_mut(&mut self) -> Option<&mut String>;
```

### Setters

For required fields, setters take the value directly:

```rust,ignore
pub fn set_name(&mut self, value: String);
pub fn set_age(&mut self, value: u32);
```

For optional fields, setters take `Option<T>`. Passing `None` removes
the field from the map:

```rust,ignore
pub fn set_email(&mut self, value: Option<String>);
```

### Removers

Only optional fields have remover methods, which remove the field from
the map and return the value if present:

```rust,ignore
pub fn remove_email(&mut self) -> Option<String>;
```

### Utility Methods

Methods for querying the struct's state:

```rust,ignore
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

```rust,ignore
#[structible]                       // Uses HashMap by default
struct Foo { /* ... */ }

#[structible(BTreeMap)]             // Shorthand for BTreeMap
struct Bar { /* ... */ }

#[structible(backing = BTreeMap)]   // Explicit key-value form
struct Baz { /* ... */ }
```

Use `BTreeMap` when you need deterministic iteration order or when field
keys should be `Ord` rather than `Hash`.

### `constructor`

**Optional.** Customizes the name of the constructor method.

```rust,ignore
#[structible(constructor = create)]
pub struct Person {
    pub name: String,
}

let person = Person::create("Alice".into());  // Instead of Person::new()
```

### Field-Level Attributes

Field-level `#[structible(...)]` attributes customize accessor method names:

| Attribute | Default | Description |
|-----------|---------|-------------|
| `get = name` | field name | Getter method name |
| `get_mut = name` | `{field}_mut` | Mutable getter method name |
| `set = name` | `set_{field}` | Setter method name |
| `remove = name` | `remove_{field}` | Remover method name (optional fields only) |
| `key = Type` | — | Marks field as unknown/extension fields catch-all |

```rust,ignore
#[structible]
pub struct Person {
    #[structible(get = full_name, set = rename)]
    pub name: String,

    #[structible(get = electronic_mail, remove = clear_email)]
    pub email: Option<String>,
}

let mut person = Person::new("Alice".into());
assert_eq!(person.full_name(), "Alice");    // Custom getter name
person.rename("Bob".into());                 // Custom setter name
person.set_email(Some("bob@example.com".into()));
person.clear_email();                        // Custom remover name
```

### Unknown/Extension Fields

The `key` attribute marks a field as a catch-all for unknown or extension
fields. This is useful for extensible data models where additional key-value
pairs may be present beyond the statically-known fields.

```rust,ignore
#[structible]
pub struct Person {
    pub name: String,
    pub age: u32,
    #[structible(key = String)]
    pub extra: Option<serde_json::Value>,
}
```

The field must be declared as `Option<T>`, where `T` is the value type for
unknown entries. The `key` attribute specifies the key type. Only one unknown
field is allowed per struct.

Unknown fields are stored in the same backing map as known fields, using an
`Unknown(K)` variant in the generated field enum.

#### Generated Methods

For a field named `extra` with key type `String` and value type `Value`:

```rust,ignore
// Insert an unknown field, returning the previous value if present
pub fn add_extra(&mut self, key: String, value: Value) -> Option<Value>;

// Get by borrowed key (e.g., &str for String keys)
pub fn extra<Q>(&self, key: &Q) -> Option<&Value>
where
    String: Borrow<Q>,
    Q: Hash + Eq + ?Sized;

// Mutable access by borrowed key
pub fn extra_mut<Q>(&mut self, key: &Q) -> Option<&mut Value>
where
    String: Borrow<Q>,
    Q: Hash + Eq + ?Sized;

// Remove and return the value
pub fn remove_extra(&mut self, key: &String) -> Option<Value>;

// Iterate all unknown fields
pub fn extra_iter(&self) -> impl Iterator<Item = (&String, &Value)>;
```

#### Example Usage

```rust,ignore
let mut person = Person::new("Alice".into(), 30);

// Add unknown fields
person.add_extra("favorite_color".into(), json!("blue"));
person.add_extra("city".into(), json!("NYC"));

// Lookup with borrowed key - no allocation needed
let color = person.extra("favorite_color");
assert_eq!(color, Some(&json!("blue")));

// Iterate all unknown fields
for (key, value) in person.extra_iter() {
    println!("{}: {}", key, value);
}

// Remove an unknown field
let removed = person.remove_extra(&"city".into());
```

## The `BackingMap` Trait

The `BackingMap` trait defines the interface required for a type to be used
as backing storage. It is implemented for `HashMap` and `BTreeMap`:

```rust
pub trait BackingMap<K, V> {
    fn new() -> Self;
    fn insert(&mut self, key: K, value: V) -> Option<V>;
    fn get(&self, key: &K) -> Option<&V>;
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;
    fn remove(&mut self, key: &K) -> Option<V>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}
```

This trait documents the contract that backing types must satisfy. Custom
map types that implement these methods can potentially be used as backing
storage.

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

### Automatic Trait Implementations

The generated struct automatically derives:
- `Debug` - for debug formatting
- `Clone` - for cloning (requires all field types to implement `Clone`)
- `PartialEq` - for equality comparison (requires all field types to implement `PartialEq`)

Additionally, `Default` is automatically implemented **only** when all fields
are optional (no required fields). Structs with required fields do not
implement `Default` because there are no sensible default values for them.

## Limitations

- Only named struct fields are supported (no tuple structs or unit structs)
- Field types must be `Clone` for the value enum to derive `Clone`
- At most one unknown/extension field per struct

## License

MIT
