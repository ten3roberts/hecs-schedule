use std::{
    any::type_name,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

/// Type alias for list of borrows
pub type Borrows = SmallVec<[Access; 8]>;

use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use smallvec::{smallvec, SmallVec};

use crate::{Access, Context, Error, Result};

use super::ComponentBorrow;

/// Wrapper type for an immutably borrowed value from schedule context
#[repr(transparent)]
#[derive(Debug)]
pub struct Read<'a, T>(pub(crate) AtomicRef<'a, T>);

impl<'a, T> Clone for Read<'a, T> {
    fn clone(&self) -> Self {
        Self(AtomicRef::<'a, T>::clone(&self.0))
    }
}

impl<'a, T> Deref for Read<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> Read<'a, T> {
    /// Creates a new Read borrow from an atomic ref
    pub fn new(borrow: AtomicRef<'a, T>) -> Self {
        Self(borrow)
    }
}

impl<'a, T: 'static> Read<'a, T> {
    pub(crate) fn try_from_untyped(cell: &'a AtomicRefCell<NonNull<u8>>) -> Result<Self> {
        cell.try_borrow()
            .map_err(|_| Error::Borrow(type_name::<T>()))
            .map(|cell| Self(AtomicRef::map(cell, |val| unsafe { val.cast().as_ref() })))
    }
}

#[repr(transparent)]
/// Wrapper type for an exclusively borrowed value
pub struct Write<'a, T>(pub(crate) AtomicRefMut<'a, T>);

impl<'a, T> Write<'a, T> {
    /// Creates a new Write borrow from an atomic ref
    pub fn new(borrow: AtomicRefMut<'a, T>) -> Self {
        Self(borrow)
    }
}

impl<'a, T: 'static> Write<'a, T> {
    pub(crate) fn try_from_untyped(cell: &'a AtomicRefCell<NonNull<u8>>) -> Result<Self> {
        cell.try_borrow_mut()
            .map_err(|_| Error::BorrowMut(type_name::<T>()))
            .map(|cell| {
                Self(AtomicRefMut::map(cell, |val| unsafe {
                    val.cast().as_mut()
                }))
            })
    }
}

impl<'a, T> Deref for Write<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for Write<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct BorrowMarker<T> {
    marker: PhantomData<T>,
}

/// Helper trait for borrowing either immutably or mutably from context
pub trait ContextBorrow<'a> {
    /// The resulting type after borrowing from context
    type Target;

    /// Borrow type from context
    fn borrow(context: &'a Context) -> Result<Self::Target>;
}

impl<'a, T: 'static> ContextBorrow<'a> for &'a T {
    type Target = Read<'a, T>;

    fn borrow(context: &'a Context) -> Result<Self::Target> {
        Read::try_from_untyped(context.cell::<&T>()?)
    }
}

impl<'a, T: 'static> ContextBorrow<'a> for &'a mut T {
    type Target = Write<'a, T>;

    fn borrow(context: &'a Context) -> Result<Self::Target> {
        Write::try_from_untyped(context.cell::<&mut T>()?)
    }
}

impl<'a, T: 'static> ContextBorrow<'a> for Read<'a, T> {
    type Target = Self;

    fn borrow(context: &'a Context) -> Result<Self::Target> {
        Read::try_from_untyped(context.cell::<&T>()?)
    }
}

impl<'a, T: 'static> ContextBorrow<'a> for Write<'a, T> {
    type Target = Self;

    fn borrow(context: &'a Context) -> Result<Self::Target> {
        Write::try_from_untyped(context.cell::<&mut T>()?)
    }
}

impl<'a, T: 'static> ComponentBorrow for Read<'a, T> {
    fn borrows() -> Borrows {
        smallvec![Access::of::<&BorrowMarker<T>>()]
    }

    fn has<U: crate::IntoAccess>() -> bool {
        Access::of::<&T>() == U::access()
    }

    fn has_dynamic(id: std::any::TypeId, exclusive: bool) -> bool {
        let l = Access::of::<&T>();

        l.id == id && !exclusive
    }
}

impl<'a, T: 'static> ComponentBorrow for Write<'a, T> {
    fn borrows() -> Borrows {
        smallvec![Access::of::<&mut BorrowMarker<T>>()]
    }

    fn has<U: crate::IntoAccess>() -> bool {
        Access::of::<&mut T>().id == U::access().id
    }

    fn has_dynamic(id: std::any::TypeId, _: bool) -> bool {
        let l = Access::of::<&mut T>();

        l.id == id
    }
}
