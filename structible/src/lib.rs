#![doc = include_str!("../README.md")]

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

pub use structible_macros::structible;

/// Trait for types that can back a structible struct.
///
/// This trait defines the operations required for a map type to be used
/// as the backing storage for a structible struct. It is implemented for
/// `HashMap` and `BTreeMap` from the standard library.
///
/// Users can implement this trait for custom map types to use them as
/// backing storage.
pub trait BackingMap<K, V> {
    /// Creates a new, empty map.
    fn new() -> Self;

    /// Inserts a key-value pair into the map, returning the previous value if present.
    fn insert(&mut self, key: K, value: V) -> Option<V>;

    /// Returns a reference to the value for the given key.
    fn get(&self, key: &K) -> Option<&V>;

    /// Returns a mutable reference to the value for the given key.
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;

    /// Removes a key from the map, returning the value if present.
    fn remove(&mut self, key: &K) -> Option<V>;

    /// Returns the number of entries in the map.
    fn len(&self) -> usize;

    /// Returns true if the map contains no entries.
    fn is_empty(&self) -> bool;
}

impl<K, V> BackingMap<K, V> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn new() -> Self {
        HashMap::new()
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        HashMap::insert(self, key, value)
    }

    fn get(&self, key: &K) -> Option<&V> {
        HashMap::get(self, key)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        HashMap::get_mut(self, key)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        HashMap::remove(self, key)
    }

    fn len(&self) -> usize {
        HashMap::len(self)
    }

    fn is_empty(&self) -> bool {
        HashMap::is_empty(self)
    }
}

impl<K, V> BackingMap<K, V> for BTreeMap<K, V>
where
    K: Ord,
{
    fn new() -> Self {
        BTreeMap::new()
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        BTreeMap::insert(self, key, value)
    }

    fn get(&self, key: &K) -> Option<&V> {
        BTreeMap::get(self, key)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        BTreeMap::get_mut(self, key)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        BTreeMap::remove(self, key)
    }

    fn len(&self) -> usize {
        BTreeMap::len(self)
    }

    fn is_empty(&self) -> bool {
        BTreeMap::is_empty(self)
    }
}

/// Extension trait for backing maps that support iteration.
///
/// This trait is required when using unknown/extension fields with
/// `#[structible(key = ...)]`. It provides `iter()` and `iter_mut()` methods
/// for iterating over entries in the map.
///
/// It is automatically implemented for `HashMap` and `BTreeMap`.
pub trait IterableMap<K, V>: BackingMap<K, V> {
    /// Iterator type for immutable iteration.
    type Iter<'a>: Iterator<Item = (&'a K, &'a V)>
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    /// Iterator type for mutable iteration.
    type IterMut<'a>: Iterator<Item = (&'a K, &'a mut V)>
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    /// Returns an iterator over all key-value pairs.
    fn iter(&self) -> Self::Iter<'_>;

    /// Returns a mutable iterator over all key-value pairs.
    fn iter_mut(&mut self) -> Self::IterMut<'_>;
}

impl<K, V> IterableMap<K, V> for HashMap<K, V>
where
    K: Eq + Hash,
{
    type Iter<'a>
        = std::collections::hash_map::Iter<'a, K, V>
    where
        K: 'a,
        V: 'a;

    type IterMut<'a>
        = std::collections::hash_map::IterMut<'a, K, V>
    where
        K: 'a,
        V: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        HashMap::iter(self)
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        HashMap::iter_mut(self)
    }
}

impl<K, V> IterableMap<K, V> for BTreeMap<K, V>
where
    K: Ord,
{
    type Iter<'a>
        = std::collections::btree_map::Iter<'a, K, V>
    where
        K: 'a,
        V: 'a;

    type IterMut<'a>
        = std::collections::btree_map::IterMut<'a, K, V>
    where
        K: 'a,
        V: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        BTreeMap::iter(self)
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        BTreeMap::iter_mut(self)
    }
}
