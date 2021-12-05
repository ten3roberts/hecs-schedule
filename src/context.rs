//! This module provides types and traits associated to accessing of borrowed
//! values.
use std::{any::TypeId, marker::PhantomData, ptr::NonNull};

use atomic_refcell::AtomicRefCell;

use crate::{borrow::ContextBorrow, Error, IntoAccess, Result};

/// Holds all data necessary for the execution of the world.
/// The data is held by references, and needs to outlive the context itself
pub struct Context<'a> {
    data: &'a dyn Data,
}

// Safe since Send + Sync is required for impl of IntoData
unsafe impl Send for Context<'_> {}
unsafe impl Sync for Context<'_> {}

impl<'a> Context<'a> {
    /// Construct a new context from the tuple of references `data`
    pub fn new(data: &'a dyn Data) -> Context {
        Self { data }
    }

    /// Borrows data of type T from the context. Does not panic.
    pub fn borrow<T>(&'a self) -> Result<T::Target>
    where
        T: ContextBorrow<'a>,
    {
        T::borrow(self)
    }

    /// Returns the cell associated to `T`.
    /// **Note**: Types are erased, but casting is guaranteed to be correct.
    pub fn cell<T: IntoAccess>(&'a self) -> Result<&AtomicRefCell<NonNull<u8>>> {
        let access = T::access();
        self.data
            .get(access.id())
            .ok_or_else(|| Error::MissingData(access.name()))
    }
}

/// Dynamically accessed static collection of values
pub unsafe trait Data {
    /// Get the cell associated to the `TypeId`.
    fn get<'a>(&'a self, ty: TypeId) -> Option<&AtomicRefCell<NonNull<u8>>>;
}

unsafe impl Data for () {
    fn get<'a>(&'a self, _: TypeId) -> Option<&AtomicRefCell<NonNull<u8>>> {
        None
    }
}

unsafe impl<A: 'static + Send + Sync> Data for (AtomicRefCell<NonNull<u8>>, PhantomData<A>) {
    fn get<'a>(&'a self, ty: TypeId) -> Option<&AtomicRefCell<NonNull<u8>>> {
        if ty == TypeId::of::<A>() {
            Some(&self.0)
        } else {
            None
        }
    }
}

/// Convert a tuple or other type into [Data].
pub trait IntoData: Send + Sync {
    /// The corresponding [Data] type.
    type Target: Data;
    /// Performs the conversion.
    unsafe fn into_data(self) -> Self::Target;
}

impl IntoData for () {
    type Target = ();

    unsafe fn into_data(self) -> Self::Target {
        ()
    }
}

macro_rules! tuple_impls {
    () => {};
    (($idx:tt => $typ:ident), $( ($nidx:tt => $ntyp:ident), )*) => {
        /*
         * Invoke recursive reversal of list that ends in the macro expansion implementation
         * of the reversed list
        */
        tuple_impls!([($idx, $typ);] $( ($nidx => $ntyp), )*);
        tuple_impls!($( ($nidx => $ntyp), )*); // invoke macro on tail
    };

        /*
     * ([accumulatedList], listToReverse); recursively calls tuple_impls until the list to reverse
     + is empty (see next pattern)
    */
    ([$(($accIdx: tt, $accTyp: ident);)+]  ($idx:tt => $typ:ident), $( ($nidx:tt => $ntyp:ident), )*) => {
      tuple_impls!([($idx, $typ); $(($accIdx, $accTyp); )*] $( ($nidx => $ntyp), ) *);
    };

    // Finally expand into the implementation
    ([($idx:tt, $typ:ident); $( ($nidx:tt, $ntyp:ident); )*]) => {
        impl<$typ, $( $ntyp ), *> IntoData for (&mut $typ, $(&mut $ntyp,) *)
            where
                $typ: 'static + Send + Sync,
                $($ntyp: 'static + Send + Sync), *
        {
            type Target = ((AtomicRefCell<NonNull<u8>>, PhantomData<$typ>), $( (AtomicRefCell<NonNull<u8>>, PhantomData<$ntyp>), )*);

            unsafe fn into_data(self) -> Self::Target {
                (
                    (AtomicRefCell::new(NonNull::new_unchecked(self.$idx as *mut _ as *mut u8)), PhantomData),
                    $( (AtomicRefCell::new(NonNull::new_unchecked(self.$nidx as *mut _ as *mut u8)), PhantomData), ) *
                )
            }

        }

        unsafe impl<$typ, $( $ntyp ), *> Data for ((AtomicRefCell<NonNull<u8>>, PhantomData<$typ>), $( (AtomicRefCell<NonNull<u8>>, PhantomData<$ntyp>), )*)
            where
                $typ: 'static,
                $($ntyp: 'static), *

        {
            fn get<'a>(&'a self, ty: TypeId) -> Option<&AtomicRefCell<NonNull<u8>>> {
                if ty == TypeId::of::<$typ>() {
                    Some(&self.$idx.0)
                } $(else if ty == TypeId::of::<$ntyp>() {
                    Some(&self.$nidx.0)
                }) *
                else {
                    None
                }
            }
        }
    };
}

tuple_impls!(
    (9 => J),
    (8 => I),
    (7 => H),
    (6 => G),
    (5 => F),
    (4 => E),
    (3 => D),
    (2 => C),
    (1 => B),
    (0 => A),
);

#[cfg(test)]
mod tests {
    use super::{Context, IntoData};

    #[test]
    fn context() {
        let mut a = 64_i32;
        let mut b = "Hello, World";

        let data = unsafe { (&mut a, &mut b).into_data() };

        let context = Context::new(&data);

        {
            let a = context.borrow::<&i32>().unwrap();
            let mut b = context.borrow::<&mut &str>().unwrap();

            assert_eq!(*b, "Hello, World");
            *b = "Foo Fighters";
            drop(b);

            let b = context.borrow::<&&str>().unwrap();
            assert_eq!(*a, 64);
            assert_eq!(*b, "Foo Fighters");

            let c = context.borrow::<&f32>();
            assert!(c.is_err());
        }
    }
}
