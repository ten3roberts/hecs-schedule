use atomic_refcell::AtomicRef;
use std::{any::type_name, marker::PhantomData, ops::Deref};

use crate::{access::*, borrow::ComponentBorrow, Error, Result};

use crate::{GenericWorld, QueryOne};
use hecs::*;
// use hecs::{Component, Entity, Query, QueryBorrow, World};

/// Type alias for a subworld referencing the world by an [atomic_refcell::AtomicRef]. Most
/// common for schedules
pub type SubWorld<'a, T> = SubWorldRaw<AtomicRef<'a, World>, T>;
/// Type alias for a subworld referencing the world by a [std::cell::Ref]
pub type SubWorldRefCell<'a, T> = SubWorldRaw<std::cell::Ref<'a, World>, T>;
/// Type alias for a subworld referencing the world by a reference
pub type SubWorldRef<'a, T> = SubWorldRaw<&'a World, T>;

/// An empty subworld, can not access any components
pub type EmptyWorld<'a> = SubWorldRef<'a, ()>;

/// Represents a borrow of the world which can only access a subset of
/// components (unless [`AllAccess`] is used).
///
/// This type allows for any reference kind, such as `&World`,
/// [AtomicRef](atomic_refcell::AtomicRef),
/// [Ref](std::cell::Ref), etc.
///
/// Type alises are provided for the most common usages, with [SubWorld] being
/// the one used by [Schedule](crate::Schedule).
pub struct SubWorldRaw<A, T> {
    pub(crate) world: A,
    marker: PhantomData<T>,
}

impl<A, T> SubWorldRaw<A, T> {
    /// Splits the world into a subworld. No borrow checking is performed so may
    /// fail during query unless guarded otherwise.
    pub fn new(world: A) -> Self {
        Self {
            world,
            marker: PhantomData,
        }
    }
}

impl<A, T: ComponentBorrow> SubWorldRaw<A, T> {
    /// Returns true if the subworld can access the borrow of T
    pub fn has<U: IntoAccess>(&self) -> bool {
        T::has::<U>()
    }

    /// Returns true if the world satisfies the whole query
    pub fn has_all<U: Subset>(&self) -> bool {
        U::is_subset::<T>()
    }
}

impl<'w, A: 'w + Deref<Target = World>, T: ComponentBorrow> SubWorldRaw<A, T> {
    /// Query the subworld.
    /// # Panics
    /// Panics if the query items are not a compatible subset of the subworld.
    pub fn query<Q: Query + Subset>(&self) -> QueryBorrow<'_, Q> {
        self.try_query()
            .expect("Failed to execute query on subworld")
    }

    /// Query the subworld for a single entity.
    /// Wraps the hecs::NoSuchEntity error and provides the entity id
    pub fn query_one<Q: Query + Subset>(&'w self, entity: Entity) -> Result<QueryOne<'w, Q>> {
        if !self.has_all::<Q>() {
            return Err(Error::IncompatibleSubworld {
                subworld: type_name::<T>(),
                query: type_name::<Q>(),
            });
        }

        let query = self
            .world
            .query_one(entity)
            .map_err(|_| Error::NoSuchEntity(entity))?;

        Ok(QueryOne::new(entity, query))
    }

    /// Get a single component from the world.
    ///
    /// Wraps the hecs::NoSuchEntity error and provides the entity id
    pub fn get<C: Component>(&self, entity: Entity) -> Result<hecs::Ref<C>> {
        if !self.has::<&C>() {
            return Err(Error::IncompatibleSubworld {
                subworld: type_name::<T>(),
                query: type_name::<&C>(),
            });
        }

        match self.world.get::<&C>(entity) {
            Ok(val) => Ok(val),
            Err(hecs::ComponentError::NoSuchEntity) => Err(Error::NoSuchEntity(entity)),
            Err(hecs::ComponentError::MissingComponent(name)) => {
                Err(Error::MissingComponent(entity, name))
            }
        }
    }

    /// Get a single component from the world.
    ///
    /// Wraps the hecs::NoSuchEntity error and provides the entity id
    pub fn get_mut<C: Component>(&self, entity: Entity) -> Result<hecs::RefMut<C>> {
        if !self.has::<&C>() {
            return Err(Error::IncompatibleSubworld {
                subworld: type_name::<T>(),
                query: type_name::<&C>(),
            });
        }

        match self.world.get::<&mut C>(entity) {
            Ok(val) => Ok(val),
            Err(hecs::ComponentError::NoSuchEntity) => Err(Error::NoSuchEntity(entity)),
            Err(hecs::ComponentError::MissingComponent(name)) => {
                Err(Error::MissingComponent(entity, name))
            }
        }
    }

    /// Reserve multiple entities concurrently
    pub fn reserve_entities(&self, count: u32) -> impl Iterator<Item = Entity> + '_ {
        self.world.reserve_entities(count)
    }

    /// Query the subworld.
    /// # Panics
    /// Panics if the query items are not a compatible subset of the subworld.
    pub fn query_par<Q: Query + Subset>(&self) -> QueryBorrow<'_, Q> {
        self.try_query()
            .expect("Failed to execute query on subworld")
    }
}
