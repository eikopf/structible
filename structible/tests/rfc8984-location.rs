use std::collections::{HashMap, HashSet};

use structible::structible;

type Link = ();
type Id = str;
type VendorStr = str;

/// A rough version of the Location type from RFC 8984 §4.2.5.
#[structible(with_len)]
struct Location<V> {
    pub name: String,
    pub description: Option<String>,
    pub location_types: Option<HashSet<String>>,
    pub relative_to: Option<String>,
    pub time_zone: Option<String>,
    pub coordinates: Option<String>,
    pub links: Option<HashMap<Box<Id>, Link>>,
    #[structible(key = Box<VendorStr>)]
    pub vendor_property: Option<V>,
}

#[test]
fn basic_usage() {
    let mut location = Location::<bool>::new("Sydney".into());
    location.set_links(Some(HashMap::new()));
    location.add_vendor_property("example.com:foo".into(), true);
    location.add_vendor_property("example.com:bar".into(), false);

    assert_eq!(location.len(), 4);
    assert_eq!(location.vendor_property("example.com:foo"), Some(&true));
    assert_eq!(location.vendor_property("example.com:bar"), Some(&false));

    if let Some(links) = location.links_mut() {
        links.insert("baz".into(), ());
        links.insert("qux".into(), ());
    }

    assert_eq!(location.name(), "Sydney");
    location.name_mut().push_str(" Harbor Bridge");
    assert_eq!(location.name(), "Sydney Harbor Bridge");
}
