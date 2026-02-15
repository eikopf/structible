use std::fmt::Debug;
use std::marker::PhantomData;

use structible::structible;

#[structible]
struct WithLifetime<'a> {
    pub name: &'a str,
    pub value: Option<&'a str>,
}

#[test]
fn test_lifetime_parameter() {
    let name = "Alice";
    let person = WithLifetime::new(name);
    assert_eq!(*person.name(), "Alice");
    assert_eq!(person.value(), None);
}

#[structible]
struct WithConstGeneric<const N: usize> {
    pub data: [u8; N],
    pub label: Option<String>,
}

#[test]
fn test_const_generic() {
    let arr = WithConstGeneric::<4>::new([1, 2, 3, 4]);
    assert_eq!(*arr.data(), [1, 2, 3, 4]);
    assert_eq!(arr.label(), None);
}

#[structible]
struct WithWhereBound<T>
where
    T: Clone + Debug,
{
    pub item: T,
    pub backup: Option<T>,
}

#[test]
fn test_where_bound() {
    let s = WithWhereBound::new("hello".to_string());
    assert_eq!(s.item(), "hello");
    assert_eq!(s.backup(), None);
}

#[structible]
struct MultipleTypes<K, V> {
    pub key: K,
    pub value: V,
    pub extra: Option<V>,
}

#[test]
fn test_multiple_type_params() {
    let m = MultipleTypes::new("id".to_string(), 42i32);
    assert_eq!(m.key(), "id");
    assert_eq!(*m.value(), 42);
}

#[structible]
struct LifetimeAndType<'a, T> {
    pub reference: &'a str,
    pub owned: T,
    pub optional_ref: Option<&'a str>,
}

#[test]
fn test_lifetime_and_type() {
    let text = "borrowed";
    let s = LifetimeAndType::new(text, 100u64);
    assert_eq!(*s.reference(), "borrowed");
    assert_eq!(*s.owned(), 100);
}

#[structible]
struct AllThree<'a, T, const N: usize>
where
    T: Default,
{
    pub name: &'a str,
    pub items: [T; N],
    pub metadata: Option<String>,
}

#[test]
fn test_all_three_combined() {
    let label = "test";
    let s = AllThree::<i32, 3>::new(label, [1, 2, 3]);
    assert_eq!(*s.name(), "test");
    assert_eq!(*s.items(), [1, 2, 3]);
}

#[structible]
struct MultipleLifetimes<'a, 'b> {
    pub first: &'a str,
    pub second: &'b str,
    pub optional: Option<&'a str>,
}

#[test]
fn test_multiple_lifetimes() {
    let a = "first";
    let b = "second";
    let s = MultipleLifetimes::new(a, b);
    assert_eq!(*s.first(), "first");
    assert_eq!(*s.second(), "second");
}

#[structible]
struct WithPhantom<T> {
    pub id: u64,
    pub marker: PhantomData<T>,
    pub name: Option<String>,
}

#[test]
fn test_phantom_data() {
    let s = WithPhantom::<String>::new(42, PhantomData);
    assert_eq!(*s.id(), 42);
}

#[structible]
struct InlineBound<T: Clone + Send> {
    pub data: T,
    pub cache: Option<T>,
}

#[test]
fn test_inline_bound() {
    let s = InlineBound::new(vec![1, 2, 3]);
    assert_eq!(s.data(), &vec![1, 2, 3]);
}

#[structible]
struct WithDefault<T = String> {
    pub content: T,
    pub alternate: Option<T>,
}

#[test]
fn test_default_type_param() {
    let s1 = WithDefault::<i32>::new(42);
    assert_eq!(*s1.content(), 42);

    let s2 = WithDefault::new("hello".to_string());
    assert_eq!(s2.content(), "hello");
}
