use std::collections::BTreeMap;
use structible::{structible, BackingMap, IterableMap};

// A simple wrapper around BTreeMap to demonstrate custom backing types
#[derive(Debug, Clone, PartialEq)]
struct MyMap<K, V> {
    inner: BTreeMap<K, V>,
}

impl<K: Ord, V> BackingMap<K, V> for MyMap<K, V> {
    fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.inner.get_mut(key)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[structible(backing = MyMap)]
pub struct Config {
    pub name: String,
    pub value: u32,
    pub description: Option<String>,
}

#[test]
fn test_custom_backing_type() {
    let mut config = Config::new("test".into(), 42);
    assert_eq!(config.name(), "test");
    assert_eq!(*config.value(), 42);
    assert_eq!(config.description(), None);

    config.set_description(Some("A test config".into()));
    assert_eq!(config.description(), Some(&"A test config".to_string()));

    *config.value_mut() = 100;
    assert_eq!(*config.value(), 100);

    assert_eq!(config.len(), 3);
    assert!(!config.is_empty());
}

#[test]
fn test_custom_backing_remove() {
    let mut config = Config::new("test".into(), 42);
    config.set_description(Some("desc".into()));

    let removed = config.remove_description();
    assert_eq!(removed, Some("desc".to_string()));
    assert_eq!(config.description(), None);
    assert_eq!(config.len(), 2);
}

// Implement IterableMap to support unknown fields
impl<K: Ord, V> IterableMap<K, V> for MyMap<K, V> {
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
        self.inner.iter()
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.inner.iter_mut()
    }
}

#[structible(backing = MyMap)]
pub struct ExtensibleConfig {
    pub name: String,
    #[structible(key = String)]
    pub extra: Option<String>,
}

#[test]
fn test_custom_backing_with_unknown_fields() {
    let mut config = ExtensibleConfig::new("test".into());
    assert_eq!(config.name(), "test");

    // Add unknown fields
    config.add_extra("color".into(), "blue".into());
    config.add_extra("size".into(), "large".into());

    // Look up by borrowed key
    assert_eq!(config.extra("color"), Some(&"blue".to_string()));
    assert_eq!(config.extra("size"), Some(&"large".to_string()));
    assert_eq!(config.extra("missing"), None);

    // Iterate unknown fields
    let entries: Vec<_> = config.extra_iter().collect();
    assert_eq!(entries.len(), 2);

    // Mutate via get_mut
    if let Some(color) = config.extra_mut("color") {
        *color = "red".into();
    }
    assert_eq!(config.extra("color"), Some(&"red".to_string()));

    // Remove
    let removed = config.remove_extra(&"size".into());
    assert_eq!(removed, Some("large".to_string()));
    assert_eq!(config.extra("size"), None);
}
