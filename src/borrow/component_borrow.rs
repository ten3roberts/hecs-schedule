use std::any::{type_name, TypeId};

use super::Borrows;
use crate::{Access, AllAccess, IntoAccess};
use hecs::{Fetch, Query, World};
pub use smallvec::smallvec;
use smallvec::SmallVec;

/// Trait for a set of component accesses
pub trait ComponentBorrow {
    /// Returns a list of all component accesses
    fn borrows() -> Borrows;
    /// Returns true if id exists in Self
    fn has_dynamic(id: TypeId, exclusive: bool) -> bool;
    /// Returns true if U exists in Self
    fn has<U: IntoAccess>() -> bool;
}

impl<'a, Q: Query> ComponentBorrow for Q {
    fn borrows() -> Borrows {
        let mut borrows = SmallVec::with_capacity(8);

        Q::Fetch::for_each_borrow(|id, exclusive| {
            borrows.push(Access::new(type_name::<Q>(), id, exclusive))
        });

        borrows
    }

    fn has_dynamic(id: TypeId, exclusive: bool) -> bool {
        let mut found = false;
        Q::Fetch::for_each_borrow(|f_id, f_exclusive| {
            if f_id == id && (!exclusive || exclusive == f_exclusive) {
                found = true
            }
        });

        found
    }

    fn has<U: IntoAccess>() -> bool {
        let u = U::access();
        Self::has_dynamic(u.id, u.exclusive)
    }
}

impl ComponentBorrow for AllAccess {
    fn borrows() -> Borrows {
        smallvec![Access::of::<&mut World>()]
    }

    // Has everything
    fn has<U: IntoAccess>() -> bool {
        true
    }

    fn has_dynamic(_: TypeId, _: bool) -> bool {
        true
    }
}
