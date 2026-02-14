# Questions
1. What are the visibility markers on the fields doing here? Are they changing the visibility of accessor methods?
2. Should the attribute explicitly set a `backing` key, or just take a single parameter without a label instead? What about leaving `HashMap` as the default, and only requiring explicit types for other maps?
3. In a related note: what is the interface we expect for map types that back our structs? Is it implicit, and could it be reified through a trait instead?

# Actions
1. Remove notes about which types are generated. That's an implementation detail, and they should be `#[doc(hidden)]` and named to avoid collision with other types as much as possible.
2. Provide a way for a user to customise generated method names (including and especially the constructor).
3. Remove direct access to the underlying map. Again, this is an implementation detail, and the user should never interact with the generated field and value types directly.

# Design Notes
1. What about contexts where some fields might have unknown names at runtime? How would we provide a way for the user to allow arbitrary additional fields, perhaps with a designated "unknown" value type.
2. What about common traits like `Debug`, `PartialEq`/`Eq`, `PartialOrd`/`Ord`, `Default`, and so on? If it is safe to implement them, I see no reason why they should not be automatically added (unless the user explicitly opts out).
