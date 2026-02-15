use structible::structible;

#[structible]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub email: Option<String>,
}

#[test]
fn test_into_fields_all_present() {
    let mut person = Person::new("Alice".into(), 30);
    person.set_email(Some("alice@example.com".into()));

    let PersonFields { name, age, email } = person.into_fields();

    assert_eq!(name, "Alice");
    assert_eq!(age, 30);
    assert_eq!(email, Some("alice@example.com".into()));
}

#[test]
fn test_into_fields_optional_missing() {
    let person = Person::new("Bob".into(), 25);

    let fields = person.into_fields();

    assert_eq!(fields.name, "Bob");
    assert_eq!(fields.age, 25);
    assert_eq!(fields.email, None);
}

#[test]
fn test_pattern_matching_destructure() {
    let mut person = Person::new("Charlie".into(), 40);
    person.set_email(Some("charlie@example.com".into()));

    let PersonFields { name, email, .. } = person.into_fields();

    assert_eq!(name, "Charlie");
    assert_eq!(email, Some("charlie@example.com".into()));
}

#[test]
fn test_take_required_field() {
    let mut person = Person::new("Diana".into(), 35);

    let name = person.take_name();
    assert_eq!(name, "Diana");

    let age = person.take_age();
    assert_eq!(age, 35);
}

#[test]
fn test_take_optional_field_present() {
    let mut person = Person::new("Eve".into(), 28);
    person.set_email(Some("eve@example.com".into()));

    let email = person.take_email();
    assert_eq!(email, Some("eve@example.com".into()));
}

#[test]
fn test_take_optional_field_absent() {
    let mut person = Person::new("Frank".into(), 50);

    let email = person.take_email();
    assert_eq!(email, None);
}

#[test]
#[should_panic(expected = "required field")]
fn test_take_required_field_twice_panics() {
    let mut person = Person::new("Grace".into(), 22);

    let _name1 = person.take_name();
    let _name2 = person.take_name(); // Should panic
}

#[test]
fn test_take_does_not_consume_struct() {
    let mut person = Person::new("Henry".into(), 45);
    person.set_email(Some("henry@example.com".into()));

    let name = person.take_name();
    assert_eq!(name, "Henry");

    // age and email still accessible
    assert_eq!(*person.age(), 45);
    assert_eq!(person.email(), Some(&"henry@example.com".into()));
}

// Generic struct test
#[structible]
pub struct Container<T> {
    pub value: T,
    pub label: Option<String>,
}

#[test]
fn test_generic_into_fields() {
    let mut container = Container::new(42i32);
    container.set_label(Some("answer".into()));

    let fields = container.into_fields();

    assert_eq!(fields.value, 42);
    assert_eq!(fields.label, Some("answer".into()));
}

#[test]
fn test_generic_take_methods() {
    let mut container = Container::new(vec![1, 2, 3]);

    let value = container.take_value();
    assert_eq!(value, vec![1, 2, 3]);
}

#[test]
fn test_generic_pattern_matching() {
    let container = Container::new("hello".to_string());

    let ContainerFields { value, .. } = container.into_fields();
    assert_eq!(value, "hello");
}

// Multiple type parameters
#[structible]
pub struct Pair<K, V> {
    pub key: K,
    pub value: V,
}

#[test]
fn test_multiple_generics_into_fields() {
    let pair = Pair::new("id".to_string(), 123u64);

    let fields = pair.into_fields();

    assert_eq!(fields.key, "id");
    assert_eq!(fields.value, 123);
}

#[test]
fn test_multiple_generics_take() {
    let mut pair = Pair::new(1, "one");

    let key = pair.take_key();
    let value = pair.take_value();

    assert_eq!(key, 1);
    assert_eq!(value, "one");
}

// Test that Fields struct derives the expected traits
#[test]
fn test_fields_struct_derives() {
    let person = Person::new("Test".into(), 20);
    let fields = person.into_fields();

    // Debug
    let _debug = format!("{:?}", fields);

    // Clone
    let cloned = fields.clone();
    assert_eq!(fields, cloned);

    // PartialEq
    assert_eq!(fields, cloned);
}

// Test field visibility in Fields struct
mod visibility_test {
    use structible::structible;

    #[structible]
    pub struct MixedVisibility {
        pub public_field: String,
        pub(crate) crate_field: u32,
        private_field: Option<bool>,
    }

    #[test]
    fn test_fields_visibility() {
        let mut item = MixedVisibility::new("hello".into(), 42);
        item.set_private_field(Some(true));

        let fields = item.into_fields();

        // Public field is accessible
        assert_eq!(fields.public_field, "hello");

        // Crate field is accessible within the same crate
        assert_eq!(fields.crate_field, 42);

        // Private field is accessible within this module
        assert_eq!(fields.private_field, Some(true));
    }
}
