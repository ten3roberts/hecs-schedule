use std::marker::PhantomData;

use crate::{access::*, Error, Result, View};
use hecs::{Query, QueryBorrow, World};

pub struct SubWorld<'a, T> {
    world: &'a World,
    marker: PhantomData<T>,
}

impl<'w, T: ComponentAccess> SubWorld<'w, T> {
    /// Splits the world into a subworld. No borrow checking is performed so may
    /// fail during query unless guarded otherwise.
    pub fn new(world: &'w World) -> Self {
        Self {
            world,
            marker: PhantomData,
        }
    }

    /// Returns true if the subworld has access the borrow of T
    pub fn has<U: IntoAccess>(&self) -> bool {
        T::has::<U>()
    }

    /// Returns true if the world satisfies the whole query
    pub fn has_all<U: Subset>(&self) -> bool {
        U::is_subset::<T>()
    }

    /// Query the subworld.
    /// # Panics
    /// Panics if the query items are not a compatible subset of the subworld.
    pub fn query<Q: Query + Subset>(&self) -> QueryBorrow<'w, Q> {
        if !self.has_all::<Q>() {
            panic!("Attempt to execute query on incompatible subworld")
        }

        self.world.query()
    }

    /// Query the subworld.
    /// Fails if the query items are not compatible with the subworld
    pub fn try_query<Q: Query + Subset + ComponentAccess>(&self) -> Result<QueryBorrow<'w, Q>> {
        if !self.has_all::<Q>() {
            return Err(Error::IncompatibleSubworld {
                subworld: T::accesses(),
                query: Q::accesses(),
            });
        } else {
            Ok(self.world.query())
        }
    }
}

impl<'a, T> View<'a> for SubWorld<'a, T>
where
    T: ComponentAccess,
{
    type Superset = World;

    fn split(world: &'a Self::Superset) -> Self {
        Self::new(world)
    }
}
