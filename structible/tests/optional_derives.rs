use structible::structible;

// Test no_clone with a non-Clone type (mutable reference)
#[structible(no_clone)]
pub struct WithMutRef<'a> {
    pub data: &'a mut i32,
}

#[test]
fn test_no_clone_with_mut_ref() {
    let mut value = 42;
    let mut obj = WithMutRef::new(&mut value);
    // data() returns &(&mut i32), so we need **
    assert_eq!(**obj.data(), 42);
    **obj.data_mut() = 100;
    assert_eq!(**obj.data(), 100);
}

// A type that implements Clone but not PartialEq
#[derive(Clone, Debug)]
pub struct CloneButNoEq(pub i32);

// Test no_partial_eq with a type that doesn't implement PartialEq
#[structible(no_partial_eq)]
pub struct WithNoEq {
    pub data: CloneButNoEq,
}

#[test]
fn test_no_partial_eq() {
    let obj = WithNoEq::new(CloneButNoEq(42));
    assert_eq!(obj.data().0, 42);

    // Verify Clone still works when only no_partial_eq is specified
    let _cloned = obj.clone();
}

// Test both flags together
#[structible(no_clone, no_partial_eq)]
pub struct WithBoth<'a> {
    pub data: &'a mut i32,
}

#[test]
fn test_both_flags() {
    let mut value = 10;
    let obj = WithBoth::new(&mut value);
    assert_eq!(**obj.data(), 10);
}

// Test combined with other flags
#[structible(no_clone, with_len)]
pub struct CombinedWithLen<'a> {
    pub data: &'a mut i32,
    pub optional: Option<String>,
}

#[test]
fn test_no_clone_with_len() {
    let mut value = 5;
    let mut obj = CombinedWithLen::new(&mut value);
    assert_eq!(obj.len(), 1); // only required field
    obj.set_optional("test".into());
    assert_eq!(obj.len(), 2);
}

// Test that existing behavior is preserved (no flags = Clone + PartialEq)
#[structible]
pub struct Normal {
    pub value: String,
}

#[test]
fn test_normal_is_clone_and_partial_eq() {
    let obj1 = Normal::new("hello".into());
    let obj2 = obj1.clone();
    assert_eq!(obj1, obj2);
}
