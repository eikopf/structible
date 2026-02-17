//! Tests for the Debug output format.
//!
//! Verifies that Debug output shows fields like a normal struct,
//! with only present fields displayed.

use structible::structible;

#[structible]
struct Person {
    name: String,
    age: u32,
    email: Option<String>,
}

#[structible]
struct AllOptional {
    first: Option<String>,
    second: Option<i32>,
    third: Option<bool>,
}

#[structible]
struct WithUnknown {
    name: String,
    #[structible(key = String)]
    extra: Option<String>,
}

#[test]
fn test_debug_shows_required_fields() {
    let person = Person::new("Alice".to_string(), 30);
    let debug_str = format!("{:?}", person);

    // Should show the struct name and present fields
    assert!(debug_str.starts_with("Person {"));
    assert!(debug_str.contains("name: \"Alice\""));
    assert!(debug_str.contains("age: 30"));
    // email is not set, should not appear
    assert!(!debug_str.contains("email"));
}

#[test]
fn test_debug_shows_optional_fields_when_present() {
    let mut person = Person::new("Bob".to_string(), 25);
    person.set_email("bob@example.com".to_string());
    let debug_str = format!("{:?}", person);

    assert!(debug_str.contains("name: \"Bob\""));
    assert!(debug_str.contains("age: 25"));
    assert!(debug_str.contains("email: \"bob@example.com\""));
}

#[test]
fn test_debug_empty_struct() {
    let empty = AllOptional::default();
    let debug_str = format!("{:?}", empty);

    // Should just show the struct name with no fields
    assert_eq!(debug_str, "AllOptional");
}

#[test]
fn test_debug_partial_fields() {
    let mut partial = AllOptional::default();
    partial.set_second(42);
    let debug_str = format!("{:?}", partial);

    assert!(debug_str.starts_with("AllOptional {"));
    assert!(debug_str.contains("second: 42"));
    assert!(!debug_str.contains("first"));
    assert!(!debug_str.contains("third"));
}

#[test]
fn test_debug_with_unknown_fields() {
    let mut item = WithUnknown::new("test".to_string());
    item.insert_extra("custom_key".to_string(), "custom_value".to_string());
    let debug_str = format!("{:?}", item);

    assert!(debug_str.contains("name: \"test\""));
    assert!(debug_str.contains("\"custom_key\": \"custom_value\""));
}

#[test]
fn test_debug_fields_struct() {
    let person = Person::new("Charlie".to_string(), 35);
    let fields = person.into_fields();
    let debug_str = format!("{:?}", fields);

    // Fields struct should also have custom Debug
    assert!(debug_str.starts_with("PersonFields {"));
    assert!(debug_str.contains("name: \"Charlie\""));
    assert!(debug_str.contains("age: 35"));
}

#[test]
fn test_debug_alternate_format() {
    let mut person = Person::new("Diana".to_string(), 28);
    person.set_email("diana@example.com".to_string());
    let debug_str = format!("{:#?}", person);

    // Alternate format should be multi-line
    assert!(debug_str.contains('\n'));
    assert!(debug_str.contains("name: \"Diana\""));
}
