use std::collections::BTreeMap;
use structible::structible;

// Test custom constructor name
#[structible(constructor = create)]
pub struct CustomConstructor {
    pub value: String,
}

#[test]
fn test_custom_constructor_name() {
    let obj = CustomConstructor::create("hello".into());
    assert_eq!(obj.value(), "hello");
}

// Test BTreeMap backing
#[structible(backing = BTreeMap)]
pub struct BTreeMapBacked {
    pub name: String,
    pub count: u32,
    pub label: Option<String>,
}

#[test]
fn test_btreemap_backing() {
    let mut obj = BTreeMapBacked::new("test".into(), 42);
    assert_eq!(obj.name(), "test");
    assert_eq!(*obj.count(), 42);
    assert_eq!(obj.label(), None);

    obj.set_label("my label".into());
    assert_eq!(obj.label(), Some(&"my label".to_string()));
}

// Test fields with function pointers (simpler than trait objects).
//
// The module-level `#[allow(...)]` suppresses a warning from the derived `PartialEq` impl
// on macro-generated types. Function pointer comparisons are unreliable because the same
// function can have different addresses across codegen units. This test only verifies that
// fn pointers work as fields and does NOT rely on comparing `WithFnPointer` instances.
#[allow(unpredictable_function_pointer_comparisons)]
mod fn_pointer_test {
    use structible::structible;

    #[structible]
    pub struct WithFnPointer {
        pub callback: fn(i32) -> i32,
    }

    #[test]
    fn test_fn_pointer_field() {
        fn double(x: i32) -> i32 {
            x * 2
        }
        let obj = WithFnPointer::new(double);
        assert_eq!((obj.callback())(5), 10);
    }
}

// Test fields with array types
#[structible]
pub struct WithArray {
    pub data: [u8; 4],
    pub optional_data: Option<[i32; 3]>,
}

#[test]
fn test_array_field() {
    let mut obj = WithArray::new([1, 2, 3, 4]);
    assert_eq!(*obj.data(), [1, 2, 3, 4]);
    assert_eq!(obj.optional_data(), None);

    obj.set_optional_data([10, 20, 30]);
    assert_eq!(obj.optional_data(), Some(&[10, 20, 30]));
}

// Test complex nested generics
#[structible]
pub struct NestedGenerics<T> {
    pub items: Vec<Option<T>>,
    pub map: std::collections::HashMap<String, Vec<T>>,
}

#[test]
fn test_nested_generics() {
    let obj = NestedGenerics::new(
        vec![Some(1), None, Some(3)],
        [("key".to_string(), vec![10, 20])].into_iter().collect(),
    );
    assert_eq!(obj.items().len(), 3);
    assert_eq!(obj.map().get("key"), Some(&vec![10, 20]));
}

// Test into_fields with all optional combinations
#[structible]
pub struct AllOptional {
    pub a: Option<String>,
    pub b: Option<i32>,
    pub c: Option<bool>,
}

#[test]
fn test_into_fields_all_none() {
    let obj = AllOptional::default();
    let mut fields = obj.into_fields();
    assert_eq!(fields.take_a(), None);
    assert_eq!(fields.take_b(), None);
    assert_eq!(fields.take_c(), None);
}

#[test]
fn test_into_fields_all_some() {
    let mut obj = AllOptional::default();
    obj.set_a("hello".into());
    obj.set_b(42);
    obj.set_c(true);

    let mut fields = obj.into_fields();
    assert_eq!(fields.take_a(), Some("hello".into()));
    assert_eq!(fields.take_b(), Some(42));
    assert_eq!(fields.take_c(), Some(true));
}

#[test]
fn test_into_fields_mixed() {
    let mut obj = AllOptional::default();
    obj.set_b(100);

    let mut fields = obj.into_fields();
    assert_eq!(fields.take_a(), None);
    assert_eq!(fields.take_b(), Some(100));
    assert_eq!(fields.take_c(), None);
}

// Test BTreeMap backing with unknown fields
#[structible(backing = BTreeMap)]
pub struct BTreeMapWithUnknown {
    pub name: String,
    #[structible(key = String)]
    pub extra: Option<String>,
}

#[test]
fn test_btreemap_with_unknown() {
    let mut obj = BTreeMapWithUnknown::new("test".into());
    obj.insert_extra("key1".into(), "value1".into());
    obj.insert_extra("key2".into(), "value2".into());

    // BTreeMap iteration should be ordered
    let entries: Vec<_> = obj.extra_iter().collect();
    assert_eq!(entries.len(), 2);
    // BTreeMap orders by key
    assert_eq!(entries[0].0, "key1");
    assert_eq!(entries[1].0, "key2");
}

// Test reference types in fields
#[structible]
pub struct WithLifetime<'a> {
    pub reference: &'a str,
    pub owned: String,
}

#[test]
fn test_lifetime_field() {
    let s = "hello";
    let obj = WithLifetime::new(s, "world".into());
    assert_eq!(*obj.reference(), "hello");
    assert_eq!(obj.owned(), "world");
}

// Test raw identifiers (e.g., using Rust keywords as field names)
#[structible]
pub struct WithRawIdentifiers {
    pub r#type: String,
    pub r#match: Option<i32>,
}

#[test]
fn test_raw_identifiers() {
    let mut obj = WithRawIdentifiers::new("keyword".into());
    assert_eq!(obj.r#type(), "keyword");
    assert_eq!(obj.r#match(), None);

    obj.set_match(42);
    assert_eq!(obj.r#match(), Some(&42));

    obj.set_type("updated".into());
    assert_eq!(obj.r#type(), "updated");
}
