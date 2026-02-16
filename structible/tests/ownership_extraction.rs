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

    let mut fields = person.into_fields();

    let name = fields.take_name().expect("required field");
    let age = fields.take_age().expect("required field");
    let email = fields.take_email();

    assert_eq!(name, "Alice");
    assert_eq!(age, 30);
    assert_eq!(email, Some("alice@example.com".into()));
}

#[test]
fn test_into_fields_optional_missing() {
    let person = Person::new("Bob".into(), 25);

    let mut fields = person.into_fields();

    assert_eq!(fields.take_name(), Some("Bob".into()));
    assert_eq!(fields.take_age(), Some(25));
    assert_eq!(fields.take_email(), None);
}

#[test]
fn test_take_fields_individually() {
    let mut person = Person::new("Charlie".into(), 40);
    person.set_email(Some("charlie@example.com".into()));

    let mut fields = person.into_fields();

    // Take just the fields we need
    let name = fields.take_name();
    let email = fields.take_email();

    assert_eq!(name, Some("Charlie".into()));
    assert_eq!(email, Some("charlie@example.com".into()));

    // Age is still available
    assert_eq!(fields.take_age(), Some(40));
}

#[test]
fn test_take_returns_none_after_first_take() {
    let person = Person::new("Diana".into(), 35);

    let mut fields = person.into_fields();

    // First take succeeds
    let name1 = fields.take_name();
    assert_eq!(name1, Some("Diana".into()));

    // Second take returns None (field was already taken)
    let name2 = fields.take_name();
    assert_eq!(name2, None);
}

#[test]
fn test_take_optional_field_present() {
    let mut person = Person::new("Eve".into(), 28);
    person.set_email(Some("eve@example.com".into()));

    let mut fields = person.into_fields();

    let email = fields.take_email();
    assert_eq!(email, Some("eve@example.com".into()));
}

#[test]
fn test_take_optional_field_absent() {
    let person = Person::new("Frank".into(), 50);

    let mut fields = person.into_fields();

    let email = fields.take_email();
    assert_eq!(email, None);
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

    let mut fields = container.into_fields();

    assert_eq!(fields.take_value(), Some(42));
    assert_eq!(fields.take_label(), Some("answer".into()));
}

#[test]
fn test_generic_take_methods() {
    let mut container = Container::new(vec![1, 2, 3]);
    container.set_label(Some("test".into()));

    let mut fields = container.into_fields();

    // Take all fields via the Fields struct
    let label = fields.take_label();
    let value = fields.take_value();

    assert_eq!(label, Some("test".into()));
    assert_eq!(value, Some(vec![1, 2, 3]));
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

    let mut fields = pair.into_fields();

    assert_eq!(fields.take_key(), Some("id".into()));
    assert_eq!(fields.take_value(), Some(123));
}

#[test]
fn test_multiple_generics_take() {
    let pair = Pair::new(1, "one");

    let mut fields = pair.into_fields();

    let key = fields.take_key();
    let value = fields.take_value();

    assert_eq!(key, Some(1));
    assert_eq!(value, Some("one"));
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

// Test field visibility - now just checks that take methods work
mod visibility_test {
    use structible::structible;

    #[structible]
    pub struct MixedVisibility {
        pub public_field: String,
        pub(crate) crate_field: u32,
        private_field: Option<bool>,
    }

    #[test]
    fn test_fields_take_visibility() {
        let mut item = MixedVisibility::new("hello".into(), 42);
        item.set_private_field(Some(true));

        let mut fields = item.into_fields();

        // All fields accessible via take_* methods within this module
        assert_eq!(fields.take_public_field(), Some("hello".into()));
        assert_eq!(fields.take_crate_field(), Some(42));
        assert_eq!(fields.take_private_field(), Some(true));
    }
}
