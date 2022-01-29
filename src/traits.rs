//! Defines common traits

use hecs::{Entity, Fetch, Query, QueryBorrow};
use rayon::iter::{ParallelBridge, ParallelIterator};

/// Traits for types which represent a view or subset of some other type.
pub trait View<'a> {
    /// The type which View comes from
    type Superset;
    /// Splits from the containing superset
    fn split(orig: Self::Superset) -> Self;
}

// Implement view for self. A set is always its own subset

impl<'a, T> View<'a> for &'a T {
    type Superset = Self;

    fn split(orig: Self::Superset) -> Self {
        orig
    }
}

impl<'a, T> View<'a> for &'a mut T {
    type Superset = Self;

    fn split(orig: Self::Superset) -> Self {
        orig
    }
}

/// Extends the queries for additional paralell operation
pub trait QueryExt {
    /// Item returned by the query
    type Item;
    /// Execute an iterator in paralell in batches
    fn par_for_each(self, batch_size: u32, func: impl Fn((Entity, Self::Item)) + Send + Sync);
    /// Execute a fallible iterator in paralell
    fn try_par_for_each<E: Send>(
        self,
        batch_size: u32,
        func: impl Fn((Entity, Self::Item)) -> Result<(), E> + Send + Sync,
    ) -> Result<(), E>;
}

impl<'w, 'q, Q> QueryExt for &'q mut QueryBorrow<'w, Q>
where
    Q: Query + Send + Sync,
{
    // type Item = <<Q as Query>::Fetch as Fetch<'q>::Item;
    // type Item = <<Q as Query>::Fetch as Fetch<'q>>::Item>;
    type Item = <Q::Fetch as Fetch<'q>>::Item;
    fn par_for_each(self, batch_size: u32, func: impl Fn((Entity, Self::Item)) + Send + Sync) {
        self.iter_batched(batch_size)
            .par_bridge()
            .for_each(|batch| batch.for_each(|val| func(val)))
    }

    fn try_par_for_each<E: Send>(
        self,
        batch_size: u32,
        func: impl Fn((Entity, Self::Item)) -> Result<(), E> + Send + Sync,
    ) -> Result<(), E> {
        self.iter_batched(batch_size)
            .par_bridge()
            .try_for_each(|mut batch| batch.try_for_each(|val| func(val)))
    }
}
