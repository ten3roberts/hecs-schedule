///! This module works around the lifetimes for borrow when GAT isn't available
use crate::{Read, SubWorld, Write};

use super::{ContextBorrow, MaybeRead, MaybeWrite};

/// Lifetime erasure in waiting of GAT
pub trait IntoBorrow {
    /// The borrow type
    type Borrow: for<'x> ContextBorrow<'x>;
}

/// Macro for implementing lifetime eliding IntoBorrow
#[macro_export]
macro_rules! impl_into_borrow {
    ($name: tt => $borrower: tt) => {
        #[doc(hidden)]
        pub struct $borrower<T>(std::marker::PhantomData<T>);

        impl<T: 'static> $crate::borrow::IntoBorrow for $name<'_, T> {
            type Borrow = $borrower<T>;
        }

        impl<'a, T: 'static> $crate::borrow::ContextBorrow<'a> for $borrower<T> {
            type Target = $name<'a, T>;

            fn borrow(context: &'a $crate::Context) -> $crate::error::Result<Self::Target> {
                Self::Target::borrow(context)
            }
        }
    };
}

impl_into_borrow!(Read => Borrower);
impl_into_borrow!(Write => BorrowMut);
impl_into_borrow!(MaybeRead => MaybeBorrower);
impl_into_borrow!(MaybeWrite => MaybeBorrowerMut);
impl_into_borrow!(SubWorld => SubWorldBorrower);
