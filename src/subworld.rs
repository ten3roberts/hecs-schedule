use atomic_refcell::AtomicRef;
use smallvec::smallvec;
use std::{any::type_name, marker::PhantomData, ops::Deref};

use crate::Context;
use crate::{
    access::*,
    borrow::{Borrows, ComponentBorrow, ContextBorrow},
    Error, Result, View,
};
use hecs::{Component, Entity, Query, QueryBorrow, QueryOne, World};

/// Type alias for a subworld referencing the world by an [atomic_refcell::AtomicRef]. Most
/// common for schedules
pub type SubWorld<'a, T> = SubWorldRaw<AtomicRef<'a, World>, T>;
/// Type alias for a subworld referencing the world by a [std::cell::Ref]
pub type SubWorldRefCell<'a, T> = SubWorldRaw<std::cell::Ref<'a, World>, T>;
/// Type alias for a subworld referencing the world by a reference
pub type SubWorldRef<'a, T> = SubWorldRaw<&'a World, T>;

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
    world: A,
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

impl<A: Deref<Target = World>, T: ComponentBorrow> SubWorldRaw<A, T> {
    /// Returns true if the subworld can access the borrow of T
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
    pub fn query<Q: Query + Subset>(&self) -> QueryBorrow<'_, Q> {
        if !self.has_all::<Q>() {
            panic!("Attempt to execute query on incompatible subworld")
        }

        self.world.query()
    }

    /// Query the subworld.
    /// Fails if the query items are not compatible with the subworld
    pub fn try_query<Q: Query + Subset>(&self) -> Result<QueryBorrow<'_, Q>> {
        if !self.has_all::<Q>() {
            return Err(Error::IncompatibleSubworld {
                subworld: T::borrows(),
                query: Q::borrows(),
            });
        } else {
            Ok(self.world.query())
        }
    }

    /// Query the subworld for a single entity.
    /// Wraps the hecs::NoSuchEntity error and provides the entity id
    pub fn try_query_one<Q: Query + Subset>(&self, entity: Entity) -> Result<QueryOne<'_, Q>> {
        if !self.has_all::<Q>() {
            return Err(Error::IncompatibleSubworld {
                subworld: T::borrows(),
                query: Q::borrows(),
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
    pub fn try_get<C: Component>(&self, entity: Entity) -> Result<hecs::Ref<C>> {
        if !self.has::<&C>() {
            return Err(Error::IncompatibleSubworld {
                subworld: T::borrows(),
                query: smallvec![Access::new::<&C>()],
            });
        }

        self.world.get(entity).map_err(|e| e.into())
    }
}

impl<A: Deref<Target = World> + Clone, T: ComponentBorrow> SubWorldRaw<A, T> {
    /// Splits the subworld further into a compatible subworld. Fails if not
    /// compatible
    pub fn split<U: ComponentBorrow + Subset>(&mut self) -> Result<SubWorldRaw<A, U>> {
        if !self.has_all::<U>() {
            return Err(Error::IncompatibleSubworld {
                subworld: T::borrows(),
                query: U::borrows(),
            });
        }

        Ok(SubWorldRaw {
            world: self.world.clone(),
            marker: PhantomData,
        })
    }
}

impl<'a, A, T> View<'a> for SubWorldRaw<A, T>
where
    A: Deref<Target = World>,
    T: ComponentBorrow,
{
    type Superset = A;

    fn split(world: Self::Superset) -> Self {
        Self::new(world)
    }
}

impl<'a, T> ContextBorrow<'a> for SubWorld<'a, T> {
    type Target = Self;

    fn borrow(context: &'a Context) -> Result<Self> {
        let val = context
            .cell::<&World>()?
            .try_borrow()
            .map_err(|_| Error::Borrow(type_name::<T>()))
            .map(|cell| AtomicRef::map(cell, |val| unsafe { val.cast().as_ref() }))?;

        Ok(Self::new(val))
    }
}

impl<'a, T> From<&'a Context<'a>> for SubWorldRaw<AtomicRef<'a, World>, T> {
    fn from(context: &'a Context) -> Self {
        let borrow = context
            .cell::<&World>()
            .expect("Failed to borrow world from context")
            .borrow();

        let val = AtomicRef::map(borrow, |val| unsafe { val.cast().as_ref() });

        Self::new(val)
    }
}

impl<A, T: ComponentBorrow> ComponentBorrow for SubWorldRaw<A, T> {
    fn borrows() -> Borrows {
        let mut access = T::borrows();
        access.push(Access::new::<&World>());
        access
    }

    fn has<U: IntoAccess>() -> bool {
        T::has::<U>()
    }
}

/// Trait for allowing function to work on both World and SubWorld
pub trait GenericWorld {
    /// Queries the world
    fn try_query<Q: Query + Subset>(&self) -> Result<QueryBorrow<Q>>;
    /// Queries the world for a specific entity
    fn try_query_one<Q: Query + Subset>(&self, entity: Entity) -> Result<QueryOne<Q>>;

    /// Get a single component for an entity
    /// Returns the contextual result since hecs-schedule is required to be imported
    /// anyway
    fn try_get<C: Component>(&self, entity: Entity) -> Result<hecs::Ref<C>>;
}

impl<A: Deref<Target = World>, T: ComponentBorrow> GenericWorld for SubWorldRaw<A, T> {
    fn try_query<Q: Query + Subset>(&self) -> Result<QueryBorrow<'_, Q>> {
        self.try_query()
    }

    fn try_query_one<Q: Query + Subset>(&self, entity: Entity) -> Result<QueryOne<'_, Q>> {
        self.try_query_one(entity)
    }

    fn try_get<C: Component>(&self, entity: Entity) -> Result<hecs::Ref<C>> {
        self.try_get(entity)
    }
}

impl GenericWorld for World {
    fn try_query<Q: Query + Subset>(&self) -> Result<QueryBorrow<Q>> {
        Ok(self.query())
    }

    fn try_query_one<Q: Query + Subset>(&self, entity: Entity) -> Result<QueryOne<Q>> {
        match self.query_one(entity) {
            Ok(val) => Ok(val),
            Err(_) => Err(Error::NoSuchEntity(entity)),
        }
    }

    fn try_get<C: Component>(&self, entity: Entity) -> Result<hecs::Ref<C>> {
        match self.get(entity) {
            Ok(val) => Ok(val),
            Err(val) => Err(Error::ComponentError(val)),
        }
    }
}
