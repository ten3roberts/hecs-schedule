//! Provides system which are an abstraction for anything that can be executed
//! against a [Context](crate::Context).
use std::{any::type_name, borrow::Cow};

use crate::{
    borrow::{Borrows, ComponentBorrow, ContextBorrow, IntoBorrow},
    Context, Result,
};

/// System name alias
pub type SystemName = Cow<'static, str>;

/// Trait which defines any function or type that can operate on a world or
/// other context.
pub trait System<Args, Ret> {
    /// Executes the by borrowing from context
    fn execute(&mut self, context: &Context) -> Result<()>;
    /// Returns the system name. Used for debug purposes
    fn name(&self) -> SystemName;

    /// Returns which data will be accessed
    fn borrows() -> Borrows;
}

macro_rules! tuple_impl {
    ($($name: ident), *) => {
        impl<Func, $($name,)  *> System<($($name,)*), ()> for Func
        where
            for<'a, 'b> &'b mut Func:
                FnMut($($name,)*) +
                FnMut($(<$name::Borrow as ContextBorrow<'a>>::Target),*),
                $($name: IntoBorrow + ComponentBorrow,)*
        {
            fn execute(&mut self, context: &Context) -> Result<()> {
                let mut func = self;
                (&mut func)($($name::Borrow::borrow(context)?), *);
                Ok(())
            }

            fn name(&self) -> SystemName {
                type_name::<Func>().into()
                // format!("System<{:?}>", ($(type_name::<$name>(),)* )).into()
            }

            fn borrows() -> Borrows {
                ([].iter()
                    $(.chain($name::borrows().iter())) *).cloned()
                .collect()
            }
        }

        impl<Err, Func, $($name,) *> System<($($name,)*), std::result::Result<(), Err>> for Func
        where
            Err: Into<anyhow::Error>,
            for<'a, 'b> &'b mut Func:
                FnMut($($name,)*) -> std::result::Result<(), Err> +
                FnMut($(<$name::Borrow as ContextBorrow<'a>>::Target),*) -> std::result::Result<(), Err>,
                $($name: IntoBorrow + ComponentBorrow,)*
        {
            fn execute(&mut self, context: &Context) -> Result<()> {
                let mut func = self;
                match (&mut func)($($name::Borrow::borrow(context)?), *) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(crate::Error::SystemError(<Self as System<($($name,)*), std::result::Result<(), Err>>>::name(func), e.into())),
                }
            }

            fn name(&self) -> SystemName {
                type_name::<Func>().into()
                // format!("System<{:?}> -> Result<(), {:?}>", ($(type_name::<$name>(),)* ), type_name::<Err>()).into()
            }

            fn borrows() -> Borrows {
                ([].iter()
                    $(.chain($name::borrows().iter())) *).cloned()
                .collect()
            }
        }
    };
}

impl<F: FnMut()> System<(), ()> for F {
    fn execute(&mut self, _: &Context) -> Result<()> {
        (self)();
        Ok(())
    }

    fn name(&self) -> SystemName {
        "System<()>".into()
    }

    fn borrows() -> Borrows {
        Borrows::default()
    }
}

impl<Err: Into<anyhow::Error>, F: FnMut() -> std::result::Result<(), Err>>
    System<(), std::result::Result<(), Err>> for F
{
    fn execute(&mut self, _: &Context) -> Result<()> {
        (self)().map_err(|e| crate::Error::SystemError(self.name(), e.into()))
    }

    fn name(&self) -> SystemName {
        "System<()>".into()
    }

    fn borrows() -> Borrows {
        Borrows::default()
    }
}

impl_for_tuples!(tuple_impl);

#[cfg(test)]
mod tests {
    use crate::{system::System, Context, IntoData, Read, SubWorld};
    use hecs::World;

    use anyhow::{ensure, Result};

    fn count_system(val: Read<i32>) {
        assert_eq!(*val, 6);
    }

    #[test]
    fn simple_system() {
        struct App {
            name: &'static str,
        }

        let mut val = 6_i32;

        let mut app = App {
            name: "hecs-schedule",
        };

        let mut world = World::default();

        let a = world.spawn(("a", 3));
        let b = world.spawn(("b", 42));
        let c = world.spawn(("c", 8));

        let data = unsafe { (&mut world, &mut app, &mut val).into_data(&mut ()) };

        let context = Context::new(&data);

        let mut count_closure = |w: SubWorld<&i32>| assert_eq!(w.query::<&i32>().iter().count(), 3);
        let mut foo = |_: SubWorld<&i32>| {};
        let mut name_query_system = |w: SubWorld<&String>| -> Result<()> {
            let name = w.get::<String>(a)?;
            eprintln!("Name: {:?}", *name);
            Ok(())
        };

        let mut name_check_system = |w: SubWorld<&&'static str>| -> anyhow::Result<()> {
            for (e, n) in [(a, "a"), (b, "b"), (c, "c")] {
                let name = w.get::<&str>(e)?;
                ensure!(*name == n, "Names did not match");
            }

            Ok(())
        };

        let mut rename_system = |w: SubWorld<&mut String>, a: Read<App>| -> anyhow::Result<()> {
            ensure!(a.name == "hecs-schedule", "App name did not match");

            w.try_query::<&mut String>()?
                .iter()
                .for_each(|(_, name)| *name = a.name.into());

            Ok(())
        };

        let mut check_rename_system = |w: SubWorld<&String>| -> anyhow::Result<()> {
            ensure!(
                w.try_query::<&String>()?
                    .iter()
                    .all(|(_, name)| name == "hecs-schedule"),
                "Names were not properly updated"
            );

            Ok(())
        };

        count_system.execute(&context).unwrap();
        foo.execute(&context).unwrap();
        count_closure.execute(&context).unwrap();
        assert!(name_query_system.execute(&context).is_err());
        name_check_system.execute(&context).unwrap();
        rename_system.execute(&context).unwrap();
        check_rename_system.execute(&context).unwrap();
    }
}
