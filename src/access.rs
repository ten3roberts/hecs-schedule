use std::any::{type_name, TypeId};

use hecs::{Fetch, Query};

use crate::borrow::ComponentBorrow;

#[derive(Copy, Clone, PartialOrd, Ord, Eq, PartialEq)]
/// Describes how a type is accessed.
pub struct Access {
    pub(crate) name: &'static str,
    pub(crate) id: TypeId,
    pub(crate) exclusive: bool,
}

impl std::fmt::Debug for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.exclusive {
            write!(f, "mut {}", self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl Access {
    /// Create a new access
    pub fn new(name: &'static str, id: TypeId, exclusive: bool) -> Self {
        Self {
            name,
            id,
            exclusive,
        }
    }

    /// Creates a new access from a known  type
    pub fn of<T: IntoAccess>() -> Self {
        T::access()
    }

    /// Get a reference to the access's exclusive.
    #[inline]
    pub fn exclusive(&self) -> bool {
        self.exclusive
    }

    pub(crate) fn id(&self) -> std::any::TypeId {
        self.id
    }

    pub(crate) fn name(&self) -> &'static str {
        self.name
    }
}

/// Convert a type into the correspodning access.
pub trait IntoAccess {
    /// Performs the conversion.
    fn access() -> Access;
    /// Check if the borrow is compatible with another borrow.
    /// A &T is compatible with &T, but not &mut T.
    /// A &mut T is compatible with &T and &mut T.
    fn compatible<U: IntoAccess>() -> bool {
        let l = Self::access();
        let r = U::access();

        l.id == r.id && (!r.exclusive || r.exclusive == l.exclusive)
    }
}

impl<T: 'static> IntoAccess for &T {
    fn access() -> Access {
        Access {
            id: TypeId::of::<T>(),
            exclusive: false,
            name: type_name::<T>(),
        }
    }
}

impl<T: 'static> IntoAccess for &mut T {
    fn access() -> Access {
        Access {
            id: TypeId::of::<T>(),
            exclusive: true,
            name: type_name::<T>(),
        }
    }
}

/// Marker type for a subworld which has access to the whole world
pub struct AllAccess;

/// Declare subset relations between tuples
pub trait Subset {
    /// Returns true if U is a subset of Self
    fn is_subset<U: ComponentBorrow>() -> bool;
}

impl<'a, Q: Query> Subset for Q {
    fn is_subset<U: ComponentBorrow>() -> bool {
        let mut all = true;
        Q::Fetch::for_each_borrow(|id, exclusive| {
            if !U::has_dynamic(id, exclusive) {
                all = false
            }
        });

        all
    }
}
