use std::{cmp::Ordering, marker::PhantomData};

use crate::{access::*, IntoComponentAccess, View};
use hecs::World;

pub struct SubWorld<'a, T> {
    world: &'a World,
    components: ComponentAccess,
    marker: PhantomData<T>,
}

impl<'a, T: IntoComponentAccess> SubWorld<'a, T> {
    /// Splits the world into a subworld. No borrow checking is performed so may
    /// fail during query unless guarded otherwise.
    pub fn new(world: &'a World) -> Self {
        let mut components = T::component_access();
        components.sort_unstable();

        Self {
            world,
            components,
            marker: PhantomData,
        }
    }

    /// Returns true if the subworld has access the borrow of T
    pub fn has<U: IntoAccess>(&self) -> bool {
        T::has::<U>()
    }
    fn has_internal(&self, access: &Access) -> bool {
        let mut low = 0;
        let mut high = 0;

        while low <= high {
            let mid = (high - low) / 2 + low;
            let val = &self.components[mid];

            match val.cmp(access) {
                Ordering::Equal => return true,
                Ordering::Less => low = mid + 1,
                Ordering::Greater => high = mid - 1,
            }
        }

        false
    }

    /// Returns true if the world satisfies the whole query
    pub fn has_all<U: IntoComponentAccess>(&self) -> bool {
        let access = U::component_access();

        access.iter().all(|val| self.has_internal(val))
    }
}

impl<'a, A> View<'a> for SubWorld<'a, A>
where
    A: IntoComponentAccess,
{
    type Superset = World;

    fn split(world: &'a Self::Superset) -> Self {
        Self::new(world)
    }
}
