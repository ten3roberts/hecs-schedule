use smallvec::smallvec;
use std::{any::type_name, marker::PhantomData};

use crate::{access::*, Error, Result, View};
use hecs::{Component, Entity, Query, QueryBorrow, QueryOne, TypeInfo, World};

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
    pub fn try_query<Q: Query + Subset>(&self) -> Result<QueryBorrow<'w, Q>> {
        if !self.has_all::<Q>() {
            return Err(Error::IncompatibleSubworld {
                subworld: T::accesses(),
                query: Q::accesses(),
            });
        } else {
            Ok(self.world.query())
        }
    }

    /// Query the subworld for a single entity.
    /// Wraps the hecs::NoSuchEntity error and provides the entity id
    pub fn query_one<Q: Query + Subset>(&self, entity: Entity) -> Result<QueryOne<'w, Q>> {
        if !self.has_all::<Q>() {
            return Err(Error::IncompatibleSubworld {
                subworld: T::accesses(),
                query: Q::accesses(),
            });
        }

        self.world
            .query_one(entity)
            .map_err(|_| Error::NoSuchEntity(entity))
    }

    /// Get a single component from the world.
    ///
    /// If a mutable borrow is desired, use [`Self::query_one`] since the world is
    /// only immutably borrowed.
    ///
    /// Wraps the hecs::NoSuchEntity error and provides the entity id
    pub fn get<C: Component>(&self, entity: Entity) -> Result<hecs::Ref<C>> {
        if !self.has::<&C>() {
            return Err(Error::IncompatibleSubworld {
                subworld: T::accesses(),
                query: smallvec![Access {
                    info: TypeInfo::of::<C>(),
                    exclusive: false,
                    name: type_name::<C>(),
                }],
            });
        }

        self.world.get(entity).map_err(|e| e.into())
    }
}

impl<'a, T> View<'a> for SubWorld<'a, T>
where
    T: ComponentAccess,
{
    type Superset = &'a World;

    fn split(world: Self::Superset) -> Self {
        Self::new(world)
    }
}
