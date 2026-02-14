use std::collections::{HashMap, HashSet};

use structible::structible;

type Link = ();
type Id = str;
type VendorStr = str;

// NOTE: this bug seems fairly simple; after running cargo-expand we find that
// the value enum type isn't being generated with a generic parameter, and the
// same also occurs with the generated Location type

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
