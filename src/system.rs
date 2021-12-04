use std::{any::type_name, borrow::Cow};

use crate::{Context, Error, Result};

/// System name alias
pub type SystemName = Cow<'static, str>;

/// Trait which defines any function or type that can operate on a world or
/// other context.
pub trait System<'a, Args> {
    /// Executes the by borrowing from context
    fn execute(&mut self, context: &'a Context) -> Result<()>;
    /// Returns the system name. Used for debug purposes
    fn name(&self) -> SystemName;
}

/// Wrapper type for fallible systems returning any result

// impl<F, E> System for Fallible<F>
// where
//     E: Into<anyhow::Error>,
//     F: FnMut(&mut World) -> std::result::Result<(), E>,
// {
//     fn execute(&mut self, context: &Context) -> Result<()> {
//         let mut a = context.borrow::<&mut World>().unwrap();
//         match (self.0)(&mut *a) {
//             Ok(()) => Ok(()),
//             Err(err) => Err(crate::Error::SystemError(self.name(), err.into())),
//         }
//     }

//     fn name(&self) -> SystemName {
//         "Fn(&mut World) -> ()".into()
//     }
// }

impl<'a, 'b, F, A> System<'a, (A,)> for F
where
    F: FnMut(A) -> (),
    A: From<&'a Context<'a>>,
{
    fn execute(&mut self, context: &'a Context) -> Result<()> {
        let foo = A::from(context);

        (self)(foo);
        Ok(())
    }

    fn name(&self) -> SystemName {
        format!("System<{:?}>", type_name::<A>(),).into()
    }
}

impl<'a, 'b, F, E, A> System<'a, (A, E)> for F
where
    E: Into<anyhow::Error>,
    F: FnMut(A) -> std::result::Result<(), E>,
    A: From<&'a Context<'a>>,
{
    fn execute(&mut self, context: &'a Context) -> Result<()> {
        let foo = A::from(context);

        (self)(foo).map_err(|e| Error::SystemError(self.name(), e.into()))
    }

    fn name(&self) -> SystemName {
        format!(
            "System<{:?}> -> Result<(), {:?}>",
            type_name::<A>(),
            type_name::<E>()
        )
        .into()
    }
}

// macro_rules! tuple_impl {
//     ($(($name: ident, $ty: ident)), *) => {
//         impl<Func, $($ty,) *> System for Func
//             where
//                 Func: FnMut($($ty), *),
//                 $($ty: for<'x> $crate::CellBorrow<'x>), *
//         {

//             fn execute(&mut self, context: & Context) -> Result<()> {
//                 $(
//                     let mut $name = context.borrow::<&mut World>().unwrap();
//                 ) *

//                 // (self)($(&mut $name) *);

//                 Ok(())
//             }

//             fn name(&self) -> SystemName {
//                 std::any::type_name::<($($ty,) *)>().into()
//             }
//         }
//     }
// }

// tuple_impl!((a, A));

#[cfg(test)]
mod tests {
    use crate::{Context, IntoData, SubWorld, System};
    use anyhow::ensure;
    use hecs::World;

    use super::Result;

    #[test]
    fn simple_system() {
        let mut world = World::default();

        let a = world.spawn(("a", 3));
        let b = world.spawn(("b", 42));
        let c = world.spawn(("c", 8));

        let data = unsafe { (&mut world,).into_data() };

        let context = Context::new(&data);

        let mut system_a = |w: SubWorld<&i32>| assert_eq!(w.query::<&i32>().iter().count(), 3);
        let mut system_b = |w: SubWorld<&String>| -> Result<()> {
            let name = w.get::<String>(a)?;
            eprintln!("Name: {:?}", *name);
            Ok(())
        };

        let mut system_c = |w: SubWorld<&&'static str>| -> anyhow::Result<()> {
            for (e, n) in [(a, "a"), (b, "b"), (c, "c")] {
                let name = w.get::<&str>(e)?;
                ensure!(*name == n, "Names did not match");
            }

            Ok(())
        };

        system_a.execute(&context).unwrap();
        assert!(system_b.execute(&context).is_err());
        system_c.execute(&context).unwrap();
    }
}
