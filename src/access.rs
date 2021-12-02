use hecs::{Component, TypeInfo};
use smallvec::{smallvec, SmallVec};

pub type ComponentAccess = SmallVec<[Access; 4]>;

#[derive(PartialOrd, Ord, Eq, PartialEq)]
pub struct Access {
    ty: TypeInfo,
    exclusive: bool,
}

impl Access {
    /// Get a reference to the access's id.
    #[inline]
    pub fn ty(&self) -> TypeInfo {
        self.ty
    }

    /// Get a reference to the access's exclusive.
    #[inline]
    pub fn exclusive(&self) -> bool {
        self.exclusive
    }
}

pub trait IntoAccess {
    fn access() -> Access;
}

impl<T: Component> IntoAccess for &T {
    fn access() -> Access {
        Access {
            ty: TypeInfo::of::<T>(),
            exclusive: false,
        }
    }
}

impl<T: Component> IntoAccess for &mut T {
    fn access() -> Access {
        Access {
            ty: TypeInfo::of::<T>(),
            exclusive: true,
        }
    }
}

pub trait IntoComponentAccess {
    fn component_access() -> ComponentAccess;
}

impl<A: IntoAccess> IntoComponentAccess for A {
    fn component_access() -> ComponentAccess {
        smallvec![A::access()]
    }
}

impl<A: IntoAccess> IntoComponentAccess for (A,) {
    fn component_access() -> ComponentAccess {
        smallvec![A::access()]
    }
}
