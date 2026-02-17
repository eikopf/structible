# structible

A proc-macro for generating map-backed structs with type-safe accessors.

```rust
use structible::structible;

#[structible]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub email: Option<String>,
}

let mut person = Person::new("Alice".into(), 30);
assert_eq!(person.name(), "Alice");
assert_eq!(*person.age(), 30);

person.set_email(Some("alice@example.com".into()));
assert_eq!(person.email(), Some(&"alice@example.com".into()));

*person.age_mut() += 1;
assert_eq!(*person.age(), 31);
```

## Quick Reference

### Struct Attributes

| Attribute | Example | Description |
|-----------|---------|-------------|
| `backing` | `#[structible(backing = BTreeMap)]` | Map type (default: `HashMap`) |
| `constructor` | `#[structible(constructor = create)]` | Constructor name (default: `new`) |
| `with_len` | `#[structible(with_len)]` | Enable `len()` and `is_empty()` methods |

### Field Attributes

| Attribute | Example | Description |
|-----------|---------|-------------|
| `get` | `#[structible(get = full_name)]` | Custom getter name |
| `get_mut` | `#[structible(get_mut = name_ref)]` | Custom mutable getter name |
| `set` | `#[structible(set = rename)]` | Custom setter name |
| `remove` | `#[structible(remove = clear)]` | Custom remover name (optional fields) |
| `key` | `#[structible(key = String)]` | Unknown/extension fields catch-all |

## Generated Methods

For each field, the macro generates:

| Field Type | Method | Signature |
|------------|--------|-----------|
| Required | Getter | `fn name(&self) -> &T` |
| Required | Mutable getter | `fn name_mut(&mut self) -> &mut T` |
| Required | Setter | `fn set_name(&mut self, value: T)` |
| Optional | Getter | `fn name(&self) -> Option<&T>` |
| Optional | Mutable getter | `fn name_mut(&mut self) -> Option<&mut T>` |
| Optional | Setter | `fn set_name(&mut self, value: Option<T>)` |
| Optional | Remover | `fn remove_name(&mut self) -> Option<T>` |
| Optional | Take | `fn take_name(&mut self) -> Option<T>` |

The constructor accepts all required fields: `fn new(name: String, age: u32) -> Self`

With `#[structible(with_len)]`:
- `fn len(&self) -> usize` — number of fields currently present
- `fn is_empty(&self) -> bool` — true if no fields are present

## BTreeMap Backing

Use `BTreeMap` for ordered iteration:

```rust,ignore
#[structible(backing = BTreeMap)]
pub struct Config {
    pub key: String,
    pub value: i32,
}
```

## Unknown/Extension Fields

Catch-all for dynamic fields beyond the statically-known ones:

```rust,ignore
#[structible]
pub struct Person {
    pub name: String,
    #[structible(key = String)]
    pub extra: Option<String>,
}

let mut person = Person::new("Alice".into());
person.add_extra("color".into(), "blue".into());
assert_eq!(person.extra("color"), Some(&"blue".into()));

for (key, value) in person.extra_iter() {
    println!("{}: {}", key, value);
}
```

Generated methods: `add_{field}`, `{field}`, `{field}_mut`, `remove_{field}`, `{field}_iter`

## Ownership Extraction

Extract owned values using `into_fields()` which returns a companion struct with `take_*` methods:

```rust,ignore
let person = Person::new("Alice".into(), 30);
let mut fields = person.into_fields();

let name = fields.take_name().expect("required field");
let age = fields.take_age().expect("required field");
let email = fields.take_email(); // None if not set
```

For optional fields, you can also use `take_*` directly on the struct without consuming it:

```rust,ignore
let mut person = Person::new("Bob".into(), 25);
person.set_email(Some("bob@example.com".into()));

let email = person.take_email(); // Some("bob@example.com")
// person.email() is now None, but person is still valid
```

Note: `take_*` methods on the main struct are only available for optional fields to prevent leaving required fields in an invalid state.

## Custom BackingMap

Implement `BackingMap<K, V>` for custom map types:

```rust
pub trait BackingMap<K, V>: Sized {
    fn new() -> Self;
    fn with_capacity(capacity: usize) -> Self { Self::new() }
    fn insert(&mut self, key: K, value: V) -> Option<V>;
    fn get(&self, key: &K) -> Option<&V>;
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;
    fn remove(&mut self, key: &K) -> Option<V>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}
```

For unknown fields support, also implement `IterableMap<K, V>`.

## Automatic Derives

Generated structs derive: `Debug`, `Clone`, `PartialEq`

`Default` is only implemented when all fields are optional.

## Limitations

- Named struct fields only (no tuple/unit structs)
- At most one unknown/extension field per struct
- Field types must implement `Clone` and `PartialEq`
