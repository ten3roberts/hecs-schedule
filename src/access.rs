use std::any::type_name;
use std::ptr::NonNull;

use crate::impl_for_tuples;
use crate::Error;
use crate::Result;
use atomic_refcell::AtomicRefMut;
use atomic_refcell::{AtomicRef, AtomicRefCell};
use hecs::TypeInfo;
use smallvec::{smallvec, SmallVec};

#[derive(Copy, Clone, PartialOrd, Ord, Eq, PartialEq)]
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
        // f.debug_struct("Access").field("name", &self.name).field("info", &self.info).field("exclusive", &self.exclusive).finish()
    }
}

impl Access {
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

pub trait IntoAccess {
    fn access() -> Access;
    fn compatible<U: IntoAccess>() -> bool;
}

impl<T: 'static> IntoAccess for &T {
    fn access() -> Access {
        Access {
            info: TypeInfo::of::<T>(),
            exclusive: false,
            name: type_name::<T>(),
        }
    }

    fn compatible<U: IntoAccess>() -> bool {
        let l = Self::access();
        let r = U::access();

        l.info == r.info && !r.exclusive
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

    fn compatible<U: IntoAccess>() -> bool {
        let l = Self::access();
        let r = U::access();

        l.info == r.info
    }
}

/// Helper trait for borrowing either immutably or mutably from an
/// AtomicRefCell.
pub trait CellBorrow<'a> {
    type Target;
    type Cell;

    fn borrow(cell: &'a AtomicRefCell<NonNull<u8>>) -> Result<Self::Target>;
}

impl<'a, T: 'static> CellBorrow<'a> for &'a T {
    type Target = AtomicRef<'a, T>;
    type Cell = Self;

    fn borrow(cell: &'a AtomicRefCell<NonNull<u8>>) -> Result<Self::Target> {
        cell.try_borrow()
            .map_err(|_| Error::Borrow(type_name::<T>()))
            .map(|cell| AtomicRef::map(cell, |val| unsafe { val.cast().as_ref() }))
    }
}

impl<'a, T: 'static> CellBorrow<'a> for &'a mut T {
    type Target = AtomicRefMut<'a, T>;
    type Cell = Self;

    fn borrow(cell: &'a AtomicRefCell<NonNull<u8>>) -> Result<Self::Target> {
        cell.try_borrow_mut()
            .map_err(|_| Error::BorrowMut(type_name::<T>()))
            .map(|cell| AtomicRefMut::map(cell, |val| unsafe { val.cast().as_mut() }))
    }
}

/// Marker type for a subworld which has access to the whole world
pub struct AllAccess;

/// Trait for a set of component accesses
pub trait ComponentAccess {
    /// Returns a list of all component accesses
    fn accesses() -> SmallVec<[Access; 8]>;
    /// Returns true if U exists in Self
    fn has<U: IntoAccess>() -> bool;
}

pub trait Subset: ComponentAccess {
    fn is_subset<U: ComponentAccess>() -> bool;
}

impl<A: IntoAccess> Subset for A {
    fn is_subset<U: ComponentAccess>() -> bool {
        U::has::<A>()
    }
}

impl<A: IntoAccess> ComponentAccess for A {
    fn accesses() -> SmallVec<[Access; 8]> {
        smallvec![A::access()]
    }
    fn has<U: IntoAccess>() -> bool {
        A::compatible::<U>()
    }
}

impl ComponentAccess for AllAccess {
    fn accesses() -> SmallVec<[Access; 8]> {
        smallvec![]
    }

    // Has everything
    fn has<U: IntoAccess>() -> bool {
        true
    }
}

/// Implement for tuples
macro_rules! tuple_impl {
    ($($name: ident), *) => {
        impl<$($name: IntoAccess,)*> ComponentAccess for ($($name,) *) {
            fn accesses() -> SmallVec<[Access; 8]> {
                smallvec![$($name::access()), *]
            }

            fn has<U: IntoAccess>() -> bool {
                $(($name::compatible::<U>())) || *
            }
        }

        impl<$($name: IntoAccess,)*> Subset for ($($name,) *) {
            fn is_subset<U: ComponentAccess>() -> bool {
                $((U::has::<$name>())) && *
            }
        }
    };

}

impl_for_tuples!(tuple_impl);
