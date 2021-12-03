use std::marker::PhantomData;

use crate::{access::*, View};
use hecs::World;

pub struct SubWorld<'a, T> {
    world: &'a World,
    marker: PhantomData<T>,
}

impl<'a, T: ComponentAccess> SubWorld<'a, T> {
    /// Splits the world into a subworld. No borrow checking is performed so may
    /// fail during query unless guarded otherwise.
    pub fn new(world: &'a World) -> Self {
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
