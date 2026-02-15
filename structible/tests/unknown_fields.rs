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

    let fields = person.into_fields();

    // Known fields are present
    assert_eq!(fields.name, "Alice");
    assert_eq!(fields.age, 30);

    // Unknown fields are collected into the extra map
    assert_eq!(fields.extra.len(), 2);
    assert_eq!(
        structible::BackingMap::get(&fields.extra, &"color".to_string()),
        Some(&"blue".to_string())
    );
    assert_eq!(
        structible::BackingMap::get(&fields.extra, &"size".to_string()),
        Some(&"medium".to_string())
    );
}
