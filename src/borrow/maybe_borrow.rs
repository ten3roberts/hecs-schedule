use std::{
    any::type_name,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use smallvec::smallvec;

use crate::{borrow::Borrows, Access, Context, Error, IntoAccess, Result};

use super::{ComponentBorrow, ContextBorrow};

/// Wrapper type for an immutably borrowed value from schedule context which may
/// not exist.
#[repr(transparent)]
#[derive(Debug)]
pub struct MaybeRead<'a, T>(pub(crate) Option<AtomicRef<'a, T>>);

// unsafe impl<T> Send for MaybeRead<'_, T> {}
// unsafe impl<T> Sync for MaybeRead<'_, T> {}

impl<'a, T> Clone for MaybeRead<'a, T> {
    fn clone(&self) -> Self {
        match &self.0 {
            Some(val) => Self(Some(AtomicRef::clone(&val))),
            None => Self(None),
        }
    }
}

impl<'a, T> Deref for MaybeRead<'a, T> {
    type Target = Option<AtomicRef<'a, T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> MaybeRead<'a, T> {
    /// Creates a new MaybeRead borrow from an atomic ref
    pub fn new(borrow: Option<AtomicRef<'a, T>>) -> Self {
        Self(borrow)
    }

    /// Returns the containing option suitable for match expressions
    pub fn option(&self) -> Option<&AtomicRef<'a, T>> {
        self.0.as_ref()
    }
}

impl<'a, T: 'static> MaybeRead<'a, T> {
    pub(crate) fn try_from_untyped(cell: Result<&'a AtomicRefCell<NonNull<u8>>>) -> Result<Self> {
        match cell {
            Ok(cell) => cell
                .try_borrow()
                .map_err(|_| Error::Borrow(type_name::<T>()))
                .map(|cell| {
                    Self(Some(AtomicRef::map(cell, |val| unsafe {
                        val.cast().as_ref()
                    })))
                }),
            Err(Error::MissingData(_)) => Ok(Self(None)),
            Err(e) => Err(e),
        }
    }
}

/// Wrapper type for an exclusively value from schedule context which may not exist.
#[repr(transparent)]
#[derive(Debug)]
pub struct MaybeWrite<'a, T>(pub(crate) Option<AtomicRefMut<'a, T>>);

impl<'a, T> Deref for MaybeWrite<'a, T> {
    type Target = Option<AtomicRefMut<'a, T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for MaybeWrite<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T> MaybeWrite<'a, T> {
    /// Creates a new MaybeWrite borrow from an atomic ref
    pub fn new(borrow: Option<AtomicRefMut<'a, T>>) -> Self {
        Self(borrow)
    }

    /// Returns the containing option suitable for match expressions
    pub fn option(&self) -> Option<&AtomicRefMut<'a, T>> {
        self.0.as_ref()
    }

    /// Returns the containing option suitable for match expressions
    pub fn option_mut(&mut self) -> Option<&mut AtomicRefMut<'a, T>> {
        self.0.as_mut()
    }
}

impl<'a, T: 'static> MaybeWrite<'a, T> {
    pub(crate) fn try_from_untyped(cell: Result<&'a AtomicRefCell<NonNull<u8>>>) -> Result<Self> {
        match cell {
            Ok(cell) => cell
                .try_borrow_mut()
                .map_err(|_| Error::BorrowMut(type_name::<T>()))
                .map(|cell| {
                    Self(Some(AtomicRefMut::map(cell, |val| unsafe {
                        val.cast().as_mut()
                    })))
                }),
            Err(Error::MissingData(_)) => Ok(Self(None)),
            Err(e) => Err(e),
        }
    }
}

struct BorrowMarker<T> {
    marker: PhantomData<T>,
}

impl<T: IntoAccess> IntoAccess for BorrowMarker<T> {
    fn access() -> Access {
        Access::of::<T>()
    }
}

impl<'a, T: 'static> ContextBorrow<'a> for MaybeRead<'a, T> {
    type Target = Self;

    fn borrow(context: &'a Context) -> Result<Self::Target> {
        MaybeRead::try_from_untyped(context.cell::<&T>())
    }
}

impl<'a, T: 'static> ContextBorrow<'a> for MaybeWrite<'a, T> {
    type Target = Self;

    fn borrow(context: &'a Context) -> Result<Self::Target> {
        MaybeWrite::try_from_untyped(context.cell::<&mut T>())
    }
}

impl<'a, T: 'static> ComponentBorrow for MaybeRead<'a, T> {
    fn borrows() -> Borrows {
        smallvec![BorrowMarker::<&T>::access()]
    }

    fn has<U: crate::IntoAccess>() -> bool {
        Access::of::<&T>() == U::access()
    }

    fn has_dynamic(id: std::any::TypeId, exclusive: bool) -> bool {
        let l = Access::of::<&T>();

        l.id == id && !exclusive
    }
}

impl<'a, T: 'static> ComponentBorrow for MaybeWrite<'a, T> {
    fn borrows() -> Borrows {
        smallvec![BorrowMarker::<&mut T>::access()]
    }

    fn has<U: crate::IntoAccess>() -> bool {
        Access::of::<&T>().id == U::access().id
    }

    fn has_dynamic(id: std::any::TypeId, _: bool) -> bool {
        let l = Access::of::<&T>();

        l.id == id
    }
}
