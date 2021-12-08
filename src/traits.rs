//! Defines common traits

/// Traits for types which represent a view or subset of some other type.
pub trait View<'a> {
    /// The type which View comes from
    type Superset;
    /// Splits from the containing superset
    fn split(orig: Self::Superset) -> Self;
}

// Implement view for self. A set is always its own subset

impl<'a, T> View<'a> for &'a T {
    type Superset = Self;

    fn split(orig: Self::Superset) -> Self {
        orig
    }
}

impl<'a, T> View<'a> for &'a mut T {
    type Superset = Self;

    fn split(orig: Self::Superset) -> Self {
        orig
    }
}
