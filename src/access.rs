use hecs::TypeInfo;
use smallvec::smallvec;
use std::any::type_name;

use crate::borrow::{Borrows, ComponentBorrow};

#[derive(Copy, Clone, PartialOrd, Ord, Eq, PartialEq)]
/// Describes how a type is accessed.
pub struct Access {
    pub(crate) name: &'static str,
    pub(crate) info: TypeInfo,
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
    /// Creates a new access from a known  type
    pub fn new<T: IntoAccess>() -> Self {
        T::access()
    }

    /// Get a reference to the access's id.
    #[inline]
    pub fn info(&self) -> TypeInfo {
        self.info
    }

    /// Get a reference to the access's exclusive.
    #[inline]
    pub fn exclusive(&self) -> bool {
        self.exclusive
    }

    pub(crate) fn id(&self) -> std::any::TypeId {
        self.info.id()
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
    fn compatible<U: IntoAccess>() -> bool {
        let l = Self::access();
        let r = U::access();

        l.info == r.info && (!r.exclusive || r.exclusive == l.exclusive)
    }
}

impl<T: 'static> IntoAccess for &T {
    fn access() -> Access {
        Access {
            info: TypeInfo::of::<T>(),
            exclusive: false,
            name: type_name::<T>(),
        }
    }
}

impl<T: 'static> IntoAccess for &mut T {
    fn access() -> Access {
        Access {
            info: TypeInfo::of::<T>(),
            exclusive: true,
            name: type_name::<T>(),
        }
    }
}

/// Marker type for a subworld which has access to the whole world
pub struct AllAccess;

///Declare subset relations between tuples
pub trait Subset: ComponentBorrow {
    /// Returns true if U is a subset of Self
    fn is_subset<U: ComponentBorrow>() -> bool;
}

impl<A: IntoAccess> Subset for A {
    fn is_subset<U: ComponentBorrow>() -> bool {
        U::has::<A>()
    }
}

impl<A: IntoAccess> ComponentBorrow for A {
    fn borrows() -> Borrows {
        smallvec![A::access()]
    }
    fn has<U: IntoAccess>() -> bool {
        A::compatible::<U>()
    }
}

impl ComponentBorrow for AllAccess {
    fn borrows() -> Borrows {
        smallvec![]
    }

    // Has everything
    fn has<U: IntoAccess>() -> bool {
        true
    }
}
