use std::collections::{HashMap, HashSet};

use structible::structible;

type Link = ();
type Id = str;
type VendorStr = str;

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
