/// Traits for types which represent a view or subset of some other type.
pub trait View<'a> {
    type Superset;
    /// Splits from the containing superset
    fn split(from: &'a Self::Superset) -> Self;
}
