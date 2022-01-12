use atomic_refcell::AtomicRef;
use std::{any::type_name, marker::PhantomData, ops::Deref};

use crate::{
    access::*,
    borrow::{Borrows, ComponentBorrow, ContextBorrow},
    traits::View,
    Error, Result,
};

use crate::{Context, QueryOne};
use hecs::{Component, Entity, Query, QueryBorrow, World};

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

        match self.world.get(entity) {
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

        match self.world.get_mut(entity) {
            Ok(val) => Ok(val),
            Err(hecs::ComponentError::NoSuchEntity) => Err(Error::NoSuchEntity(entity)),
            Err(hecs::ComponentError::MissingComponent(name)) => {
                Err(Error::MissingComponent(entity, name))
            }
        }
    }

    /// Reserve multiple entities concurrently
    pub fn reserve_entities<'a>(&'a self, count: u32) -> impl Iterator<Item = Entity> + 'a {
        self.world.reserve_entities(count)
    }
}

impl<A: Deref<Target = World>, T: Query> SubWorldRaw<A, T> {
    /// Query the full access of the subworld. Does not fail as access is
    /// guaranteed
    pub fn native_query(&self) -> QueryBorrow<'_, T> {
        self.world.query::<T>()
    }
}

impl<A: ExternalClone, T: ComponentBorrow> SubWorldRaw<A, T> {
    /// Splits the subworld further into a compatible subworld. Fails if not
    /// compatible
    pub fn split<U: ComponentBorrow + Subset>(&self) -> Result<SubWorldRaw<A, U>> {
        if !self.has_all::<U>() {
            return Err(Error::IncompatibleSubworld {
                subworld: type_name::<T>(),
                query: type_name::<SubWorldRaw<A, U>>(),
            });
        }

        Ok(SubWorldRaw {
            world: A::external_clone(&self.world),
            marker: PhantomData,
        })
    }
}

/// Helper trait for types which do not implement clone, but has a clone wrapper
pub trait ExternalClone {
    /// Clones the internal value
    fn external_clone(&self) -> Self;
}

impl<T> ExternalClone for &T {
    fn external_clone(&self) -> Self {
        self.clone()
    }
}

impl<T> ExternalClone for std::cell::Ref<'_, T> {
    fn external_clone(&self) -> Self {
        std::cell::Ref::clone(self)
    }
}

impl<T> ExternalClone for AtomicRef<'_, T> {
    fn external_clone(&self) -> Self {
        AtomicRef::clone(self)
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

impl<A: ExternalClone, T: ComponentBorrow, U: ComponentBorrow + Subset> From<&SubWorldRaw<A, T>>
    for SubWorldRaw<A, U>
{
    fn from(value: &SubWorldRaw<A, T>) -> Self {
        value.split().expect("Incompatible subworld")
    }
}

impl<A, T> From<A> for SubWorldRaw<A, T> {
    fn from(world: A) -> Self {
        Self::new(world)
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

impl<'a, 'b, T: ComponentBorrow, U: ComponentBorrow + Subset> From<&'b SubWorld<'a, T>>
    for SubWorldRef<'b, U>
{
    fn from(subworld: &'b SubWorld<'a, T>) -> Self {
        let world = subworld.world.deref();
        SubWorldRef::<T>::new(world)
            .split()
            .expect("Incompatible subworld")
    }
}

impl<A, T: ComponentBorrow + Query> ComponentBorrow for SubWorldRaw<A, T> {
    fn borrows() -> Borrows {
        let mut access = T::borrows();
        access.push(Access::of::<&World>());
        access
    }

    fn has<U: IntoAccess>() -> bool {
        T::has::<U>()
    }

    fn has_dynamic(id: std::any::TypeId, exclusive: bool) -> bool {
        T::has_dynamic(id, exclusive)
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

    /// Get a single component for an entity
    /// Returns the contextual result since hecs-schedule is required to be imported
    /// anyway
    fn try_get_mut<C: Component>(&self, entity: Entity) -> Result<hecs::RefMut<C>>;

    /// Reserve an entity
    fn reserve(&self) -> Entity;
}

impl<A: Deref<Target = World>, T: ComponentBorrow> GenericWorld for SubWorldRaw<A, T> {
    fn try_query<Q: Query + Subset>(&self) -> Result<QueryBorrow<'_, Q>> {
        if !self.has_all::<Q>() {
            return Err(Error::IncompatibleSubworld {
                subworld: type_name::<T>(),
                query: type_name::<Q>(),
            });
        } else {
            Ok(self.world.query())
        }
    }

    fn try_query_one<Q: Query + Subset>(&self, entity: Entity) -> Result<QueryOne<'_, Q>> {
        self.query_one(entity)
    }

    fn try_get<C: Component>(&self, entity: Entity) -> Result<hecs::Ref<C>> {
        self.get(entity)
    }

    fn try_get_mut<C: Component>(&self, entity: Entity) -> Result<hecs::RefMut<C>> {
        self.get_mut(entity)
    }

    /// Reserve an entity
    fn reserve(&self) -> Entity {
        self.world.reserve_entity()
    }
}

impl GenericWorld for World {
    fn try_query<Q: Query + Subset>(&self) -> Result<QueryBorrow<Q>> {
        Ok(self.query())
    }

    fn try_query_one<Q: Query + Subset>(&self, entity: Entity) -> Result<QueryOne<Q>> {
        match self.query_one(entity) {
            Ok(val) => Ok(QueryOne::new(entity, val)),
            Err(_) => Err(Error::NoSuchEntity(entity)),
        }
    }

    fn try_get<C: Component>(&self, entity: Entity) -> Result<hecs::Ref<C>> {
        match self.get(entity) {
            Ok(val) => Ok(val),
            Err(hecs::ComponentError::NoSuchEntity) => Err(Error::NoSuchEntity(entity)),
            Err(hecs::ComponentError::MissingComponent(name)) => {
                Err(Error::MissingComponent(entity, name))
            }
        }
    }

    fn try_get_mut<C: Component>(&self, entity: Entity) -> Result<hecs::RefMut<C>> {
        match self.get_mut(entity) {
            Ok(val) => Ok(val),
            Err(hecs::ComponentError::NoSuchEntity) => Err(Error::NoSuchEntity(entity)),
            Err(hecs::ComponentError::MissingComponent(name)) => {
                Err(Error::MissingComponent(entity, name))
            }
        }
    }

    /// Reserve an entity
    fn reserve(&self) -> Entity {
        self.reserve_entity()
    }
}
