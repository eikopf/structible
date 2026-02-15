use std::collections::{HashMap, HashSet};

use structible::structible;

type Link = ();
type Id = str;
type VendorStr = str;

/// A rough version of the Location type from RFC 8984 §4.2.5.
#[structible]
struct Location<V> {
    pub name: String,
    pub description: Option<String>,
    pub location_types: HashSet<String>,
    pub relative_to: Option<String>,
    pub time_zone: Option<String>,
    pub coordinates: Option<String>,
    pub links: HashMap<Box<Id>, Link>,
    #[structible(key = Box<VendorStr>)]
    pub vendor_property: Option<V>,
}
