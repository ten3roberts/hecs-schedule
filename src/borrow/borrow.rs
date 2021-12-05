use std::{
    any::type_name,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub type Borrows = SmallVec<[Access; 4]>;

use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use smallvec::{smallvec, SmallVec};

use crate::{Access, ComponentBorrow, Context, Error, Result};

/// Wrapper type for an immutably borrowed value from schedule context
#[repr(transparent)]
#[derive(Debug)]
pub struct Borrow<'a, T>(pub(crate) AtomicRef<'a, T>);

impl<'a, T> Clone for Borrow<'a, T> {
    fn clone(&self) -> Self {
        Self(AtomicRef::<'a, T>::clone(&self.0))
    }
}

impl<'a, T> Deref for Borrow<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> Borrow<'a, T> {
    pub fn new(borrow: AtomicRef<'a, T>) -> Self {
        Self(borrow)
    }
}

impl<'a, T: 'static> Borrow<'a, T> {
    pub(crate) fn try_from_untyped(cell: &'a AtomicRefCell<NonNull<u8>>) -> Result<Self> {
        cell.try_borrow()
            .map_err(|_| Error::Borrow(type_name::<T>()))
            .map(|cell| Self(AtomicRef::map(cell, |val| unsafe { val.cast().as_ref() })))
    }
}

#[repr(transparent)]
/// Wrapper type for an immutably borrowed value
pub struct BorrowMut<'a, T>(pub(crate) AtomicRefMut<'a, T>);

impl<'a, T> BorrowMut<'a, T> {
    pub fn new(borrow: AtomicRefMut<'a, T>) -> Self {
        Self(borrow)
    }
}

impl<'a, T: 'static> BorrowMut<'a, T> {
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

impl<'a, T> Deref for BorrowMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for BorrowMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Helper trait for borrowing either immutably or mutably from context
pub trait ContextBorrow<'a> {
    type Target;

    fn borrow(context: &'a Context) -> Result<Self::Target>;
}

impl<'a, T: 'static> ContextBorrow<'a> for &'a T {
    type Target = Borrow<'a, T>;

    fn borrow(context: &'a Context) -> Result<Self::Target> {
        Borrow::try_from_untyped(context.cell::<&T>()?)
    }
}

impl<'a, T: 'static> ContextBorrow<'a> for &'a mut T {
    type Target = BorrowMut<'a, T>;

    fn borrow(context: &'a Context) -> Result<Self::Target> {
        BorrowMut::try_from_untyped(context.cell::<&mut T>()?)
    }
}

impl<'a, T: 'static> ContextBorrow<'a> for Borrow<'a, T> {
    type Target = Self;

    fn borrow(context: &'a Context) -> Result<Self::Target> {
        Borrow::try_from_untyped(context.cell::<&T>()?)
    }
}

impl<'a, T: 'static> ContextBorrow<'a> for BorrowMut<'a, T> {
    type Target = Self;

    fn borrow(context: &'a Context) -> Result<Self::Target> {
        BorrowMut::try_from_untyped(context.cell::<&mut T>()?)
    }
}

impl<'a, T: 'static> ComponentBorrow for Borrow<'a, T> {
    fn borrows() -> crate::Borrows {
        smallvec![Access::new::<&T>()]
    }

    fn has<U: crate::IntoAccess>() -> bool {
        let l = Access::new::<&T>();
        let r = U::access();

        l.info == r.info && !r.exclusive
    }
}

impl<'a, T: 'static> ComponentBorrow for BorrowMut<'a, T> {
    fn borrows() -> crate::Borrows {
        smallvec![Access::new::<&mut T>()]
    }

    fn has<U: crate::IntoAccess>() -> bool {
        let l = Access::new::<&mut T>();
        let r = U::access();

        l.info == r.info
    }
}
