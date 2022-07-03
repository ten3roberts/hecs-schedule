//! This module provides types and traits associated to accessing of borrowed
//! values.
use std::{any::TypeId, cmp::Ordering, ptr::NonNull};

use atomic_refcell::AtomicRefCell;

use crate::{borrow::ContextBorrow, Error, IntoAccess, Result};
use hecs::Component;

/// Holds all data necessary for the execution of the world.
/// The data is held by references, and needs to outlive the context itself
pub struct Context<'a> {
    data: &'a dyn Data,
}

// Safe since Send + Sync is required for impl of IntoData
unsafe impl Send for Context<'_> {}
unsafe impl Sync for Context<'_> {}

mod erased_cell;
use erased_cell::*;

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
pub trait Data {
    /// Get the cell associated to the `TypeId`.
    fn get(&self, ty: TypeId) -> Option<&AtomicRefCell<NonNull<u8>>>;
}

/// Convert a tuple or other type into [Data].
pub trait IntoData<With>: Send + Sync {
    /// The corresponding [Data] type.
    type Target: Data;
    /// Performs the conversion.
    /// # Safety
    /// Converts a tuple of references into NonNull. The lifetime is captures by
    /// [Context]
    unsafe fn into_data(self, with: &mut With) -> Self::Target;
}

impl<With: Component> IntoData<With> for () {
    type Target = [ErasedCell; 1];

    unsafe fn into_data(self, with: &mut With) -> Self::Target {
        [ErasedCell::from_ref(with)]
    }
}

macro_rules! tuple_impl {
    ($([$idx: tt => $name: ident]),*) => {
        impl<$( $name ), *, With> IntoData<With> for ($(&mut $name,) *)
            where
                With: Component,
                $($name: Component), *
        {
            type Target = [ErasedCell; 1 + count!($($name )*)];

            unsafe fn into_data(self, with: &mut With) -> Self::Target {

                let mut val = [
                    $( ErasedCell::from_ref::<$name>(self.$idx),)*
                    ErasedCell::from_ref(with),
                ];

                val.sort_unstable();

                val
            }
        }
    };
}

impl_for_tuples_idx!(tuple_impl);

impl<const C: usize> Data for [ErasedCell; C] {
    fn get(&self, ty: TypeId) -> Option<&AtomicRefCell<NonNull<u8>>> {
        let mut low = 0;
        let mut high = C - 1;

        while low <= high {
            let mid = (high - low) / 2 + low;
            let val = &self[mid];

            match val.cmp_id(ty) {
                Ordering::Less => low = mid + 1,
                Ordering::Equal => return Some(&val.cell),
                Ordering::Greater if mid == 0 => break,
                Ordering::Greater => high = mid - 1,
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::{Context, IntoData};

    #[test]
    fn context() {
        let mut a = 64_i32;
        let mut b = "Hello, World";

        let data = unsafe { (&mut a, &mut b).into_data(&mut ()) };

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
