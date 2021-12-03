use crate::impl_for_tuples;
use hecs::TypeInfo;

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

impl<T: 'static> IntoAccess for &T {
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

impl<T: 'static> IntoAccess for &mut T {
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

pub trait ComponentAccess {
    fn has<U: IntoAccess>() -> bool;
}

pub trait Subset {
    fn is_subset<U: ComponentAccess>() -> bool;
}

impl<A: IntoAccess> Subset for A {
    fn is_subset<U: ComponentAccess>() -> bool {
        U::has::<A>()
    }
}

impl<A: IntoAccess> ComponentAccess for A {
    fn has<U: IntoAccess>() -> bool {
        A::compatible::<U>()
    }
}

/// Implement for tuples
macro_rules! tuple_impl {
    ($($name: ident), *) => {
        impl<$($name: IntoAccess,)*> ComponentAccess for ($($name,) *) {
            fn has<U: IntoAccess>() -> bool {
                $(($name::compatible::<U>())) || *
            }
        }

        impl<$($name: IntoAccess,)*> Subset for ($($name,) *) {
            fn is_subset<U: ComponentAccess>() -> bool {
                $((U::has::<$name>())) && *
            }
        }

    };

}

impl_for_tuples!(tuple_impl);
