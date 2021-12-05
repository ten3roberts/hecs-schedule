use super::Borrows;
use crate::{IntoAccess, Subset};
pub use smallvec::smallvec;

/// Trait for a set of component accesses
pub trait ComponentBorrow {
    /// Returns a list of all component accesses
    fn borrows() -> Borrows;
    /// Returns true if U exists in Self
    fn has<U: IntoAccess>() -> bool;
}

/// Implement for tuples
macro_rules! tuple_impl {
    ($($name: ident), *) => {
        impl<$($name: IntoAccess,)*> ComponentBorrow for ($($name,) *) {
            fn borrows() -> Borrows {
                smallvec![$($name::access()), *]
            }

            fn has<U: IntoAccess>() -> bool {
                $(($name::compatible::<U>())) || *
            }
        }

        impl<$($name: IntoAccess,)*> Subset for ($($name,) *) {
            fn is_subset<U: ComponentBorrow>() -> bool {
                $((U::has::<$name>())) && *
            }
        }
    };

}

impl_for_tuples!(tuple_impl);
