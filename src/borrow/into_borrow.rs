///! This module works around the lifetimes for borrow when GAT isn't available
use std::marker::PhantomData;

use crate::{Context, Read, Result, SubWorld, Write};

use super::ContextBorrow;

pub struct Borrower<T>(PhantomData<T>);

// Lifetime erasure in waiting of GAT
pub trait IntoBorrow {
    type Borrow: for<'x> ContextBorrow<'x>;
}

impl<T: 'static> IntoBorrow for Read<'_, T> {
    type Borrow = Borrower<T>;
}

impl<'a, T: 'static> ContextBorrow<'a> for Borrower<T> {
    type Target = Read<'a, T>;

    fn borrow(context: &'a Context) -> Result<Self::Target> {
        Self::Target::borrow(context)
    }
}

pub struct BorrowerMut<T>(PhantomData<T>);

impl<T: 'static> IntoBorrow for Write<'_, T> {
    type Borrow = BorrowerMut<T>;
}

impl<'a, T: 'static> ContextBorrow<'a> for BorrowerMut<T> {
    type Target = Write<'a, T>;

    fn borrow(context: &'a Context) -> Result<Self::Target> {
        Self::Target::borrow(context)
    }
}

pub struct SubWorldBorrower<T>(PhantomData<T>);

impl<T: 'static> IntoBorrow for SubWorld<'_, T> {
    type Borrow = SubWorldBorrower<T>;
}

impl<'a, T: 'static> ContextBorrow<'a> for SubWorldBorrower<T> {
    type Target = SubWorld<'a, T>;

    fn borrow(context: &'a Context) -> Result<Self::Target> {
        Self::Target::borrow(context)
    }
}
