use structible::structible;

#[structible]
pub struct Person {
    pub name: String,
    pub age: u32,
    #[structible(key = String)]
    pub extra: Option<String>,
}

#[test]
fn test_basic_fields() {
    let person = Person::new("Alice".into(), 30);
    assert_eq!(person.name(), "Alice");
    assert_eq!(*person.age(), 30);
}

#[test]
fn test_add_unknown_field() {
    let mut person = Person::new("Alice".into(), 30);

    // Add an unknown field
    let prev = person.add_extra("favorite_color".into(), "blue".into());
    assert!(prev.is_none());

    // Add another
    person.add_extra("city".into(), "NYC".into());
}

#[test]
fn test_get_unknown_field() {
    let mut person = Person::new("Alice".into(), 30);
    person.add_extra("favorite_color".into(), "blue".into());

    // Get by borrowed key
    let color = person.extra("favorite_color");
    assert_eq!(color, Some(&"blue".to_string()));

    // Non-existent key
    let missing = person.extra("nonexistent");
    assert!(missing.is_none());
}

#[test]
fn test_get_mut_unknown_field() {
    let mut person = Person::new("Alice".into(), 30);
    person.add_extra("score".into(), "100".into());

    // Mutate via get_mut
    if let Some(score) = person.extra_mut("score") {
        *score = "200".into();
    }

    assert_eq!(person.extra("score"), Some(&"200".to_string()));
}

#[test]
fn test_remove_unknown_field() {
    let mut person = Person::new("Alice".into(), 30);
    person.add_extra("temp".into(), "value".into());

    let removed = person.remove_extra("temp");
    assert_eq!(removed, Some("value".to_string()));

    // Should be gone now
    assert!(person.extra("temp").is_none());
}

#[test]
fn test_remove_with_borrowed_str() {
    let mut person = Person::new("Alice".into(), 30);
    person.add_extra("key1".into(), "value1".into());
    person.add_extra("key2".into(), "value2".into());

    // Remove using &str directly (the ergonomic API)
    let removed = person.remove_extra("key1");
    assert_eq!(removed, Some("value1".to_string()));

    // Remove using a str slice from another string
    let key = String::from("key2");
    let removed = person.remove_extra(key.as_str());
    assert_eq!(removed, Some("value2".to_string()));
}

#[test]
fn test_remove_with_owned_string_ref() {
    let mut person = Person::new("Alice".into(), 30);
    person.add_extra("mykey".into(), "myvalue".into());

    // Remove using &String (backwards compatibility)
    let key = String::from("mykey");
    let removed = person.remove_extra(&key);
    assert_eq!(removed, Some("myvalue".to_string()));
}

#[test]
fn test_remove_nonexistent_key() {
    let mut person = Person::new("Alice".into(), 30);
    person.add_extra("exists".into(), "value".into());

    // Removing a key that doesn't exist returns None
    let removed = person.remove_extra("does_not_exist");
    assert!(removed.is_none());

    // Original key still present
    assert_eq!(person.extra("exists"), Some(&"value".to_string()));
}

#[test]
fn test_remove_same_key_twice() {
    let mut person = Person::new("Alice".into(), 30);
    person.add_extra("once".into(), "only".into());

    // First removal succeeds
    let first = person.remove_extra("once");
    assert_eq!(first, Some("only".to_string()));

    // Second removal returns None
    let second = person.remove_extra("once");
    assert!(second.is_none());
}

#[test]
fn test_iterate_unknown_fields() {
    let mut person = Person::new("Alice".into(), 30);
    person.add_extra("a".into(), "1".into());
    person.add_extra("b".into(), "2".into());

    let entries: Vec<_> = person.extra_iter().collect();
    assert_eq!(entries.len(), 2);
}

#[test]
fn test_into_fields_with_unknown() {
    let mut person = Person::new("Alice".into(), 30);
    person.add_extra("color".into(), "blue".into());
    person.add_extra("size".into(), "medium".into());

    let mut fields = person.into_fields();

    // Known fields accessible via take_* methods
    assert_eq!(fields.take_name(), Some("Alice".into()));
    assert_eq!(fields.take_age(), Some(30));

    // Unknown fields accessible via take_extra or extra_iter
    let entries: Vec<_> = fields.extra_iter().collect();
    assert_eq!(entries.len(), 2);

    // Take individual unknown fields
    assert_eq!(fields.take_extra("color"), Some("blue".to_string()));
    assert_eq!(fields.take_extra("size"), Some("medium".to_string()));

    // After taking, they're gone
    assert_eq!(fields.take_extra("color"), None);
}

#[test]
fn test_into_fields_drain_unknown() {
    let mut person = Person::new("Bob".into(), 25);
    person.add_extra("key1".into(), "val1".into());
    person.add_extra("key2".into(), "val2".into());

    let mut fields = person.into_fields();

    // Drain all unknown fields at once
    let extra = fields.drain_extra();
    assert_eq!(extra.len(), 2);
    assert_eq!(
        structible::BackingMap::get(&extra, &"key1".to_string()),
        Some(&"val1".to_string())
    );
    assert_eq!(
        structible::BackingMap::get(&extra, &"key2".to_string()),
        Some(&"val2".to_string())
    );

    // After drain, extra_iter returns nothing
    assert_eq!(fields.extra_iter().count(), 0);
}
