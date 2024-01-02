//! Defines common traits
use hecs::{Query, QueryBorrow};

#[cfg(feature = "parallel")]
use hecs::Entity;

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
    type Item<'a>;
    /// Execute a function for each item of the query in pararell using rayon.
    #[cfg(feature = "parallel")]
    fn par_for_each<'a>(self, batch_size: u32, func: impl Fn((Entity, Self::Item<'a>)) + Send + Sync);
    /// Fallible version of [`QueryBorrow::par_for_each`]
    #[cfg(feature = "parallel")]
    fn try_par_for_each<'a, E: Send>(
        self,
        batch_size: u32,
        func: impl Fn((Entity, Self::Item<'a>)) -> Result<(), E> + Send + Sync,
    ) -> Result<(), E>;
}

impl<'w, 'q, Q> QueryExt for &'q mut QueryBorrow<'w, Q>
where
    Q: Query,
    for <'a> Q::Item<'a>: Send,
{
    type Item<'a> = Q::Item<'q>;

    #[cfg(feature = "parallel")]
    fn par_for_each<'a>(self, batch_size: u32, func: impl Fn((Entity, Self::Item<'a>)) + Send + Sync) {
        use rayon::iter::{ParallelBridge, ParallelIterator};
        self.iter_batched(batch_size)
            .par_bridge()
            .for_each(|batch| batch.for_each(&func))
    }

    #[cfg(feature = "parallel")]
    fn try_par_for_each<'a, E: Send>(
        self,
        batch_size: u32,
        func: impl Fn((Entity, Self::Item<'a>)) -> Result<(), E> + Send + Sync,
    ) -> Result<(), E> {
        use rayon::iter::{ParallelBridge, ParallelIterator};
        self.iter_batched(batch_size)
            .par_bridge()
            .try_for_each(|mut batch| batch.try_for_each(&func))
    }
}
