use crate::impl_for_tuples;
use hecs::{Component, TypeInfo};
use smallvec::{smallvec, SmallVec};

pub type ComponentAccess = SmallVec<[Access; 8]>;

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
    fn compatible<U: IntoAccess>() -> bool;
}

impl<T: Component> IntoAccess for &T {
    fn access() -> Access {
        Access {
            ty: TypeInfo::of::<T>(),
            exclusive: false,
        }
    }

    fn compatible<U: IntoAccess>() -> bool {
        let l = Self::access();
        let r = U::access();

        l.ty == r.ty && !r.exclusive
    }
}

impl<T: Component> IntoAccess for &mut T {
    fn access() -> Access {
        Access {
            ty: TypeInfo::of::<T>(),
            exclusive: true,
        }
    }

    fn compatible<U: IntoAccess>() -> bool {
        let l = Self::access();
        let r = U::access();

        l.ty == r.ty
    }
}

pub trait IntoComponentAccess {
    fn component_access() -> ComponentAccess;
    fn has<U: IntoAccess>() -> bool;
}

impl<A: IntoAccess> IntoComponentAccess for A {
    fn component_access() -> ComponentAccess {
        smallvec![A::access()]
    }

    fn has<U: IntoAccess>() -> bool {
        A::compatible::<U>()
    }
}

// impl<A: IntoAccess> IntoComponentAccess for (A,) {
//     fn component_access() -> ComponentAccess {
//         smallvec![A::access()]
//     }

//     fn has<U: IntoAccess>() -> bool {
//         A::compatible::<U>()
//     }
// }

/// Implement for tuples
macro_rules! tuple_impl {
    ($($name: ident), *) => {
        impl<$($name: IntoAccess,)*> IntoComponentAccess for ($($name,) *) {
            fn component_access() -> ComponentAccess {
                smallvec![$($name::access()), *]
            }

            fn has<U: IntoAccess>() -> bool {
                $(($name::compatible::<U>())) || *
            }
        }
    };

}

impl_for_tuples!(tuple_impl);
