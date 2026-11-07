// A domain model: immutable once made. Its fields are public to read; nothing in the domain container holds a
// `mut` one, and Rust's borrowing rules keep a value another component holds from changing under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Customer {
    pub id: String,
    pub name: String,
    pub email: String,
}
