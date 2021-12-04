use std::{any::type_name, borrow::Cow};

use crate::{Context, ContextBorrow, Error, Result};

/// System name alias
pub type SystemName = Cow<'static, str>;

/// Trait which defines any function or type that can operate on a world or
/// other context.
pub trait System<'a, Args, Ret> {
    /// Executes the by borrowing from context
    fn execute(&mut self, context: &'a Context) -> Result<()>;
    /// Returns the system name. Used for debug purposes
    fn name(&self) -> SystemName;
}

macro_rules! tuple_impl {
    ($($name: ident), *) => {
        impl<'a, 'b, Func, $($name,)  *> System<'a, ($($name,)*), ()> for Func
        where
            Func: FnMut($($name),*) -> (),
            $($name: ContextBorrow<'a, Target = $name>,)*
        {
            fn execute(&mut self, context: &'a Context) -> Result<()> {
                (self)($($name::borrow(context)?), *);
                Ok(())
            }

            fn name(&self) -> SystemName {
                format!("System<{:?}>", type_name::<A>(),).into()
            }
        }

        impl<'a, 'b, Err, Func, $($name,)  *> System<'a, ($($name,)*), std::result::Result<(), Err>> for Func
        where
            Err: Into<anyhow::Error>,
            Func: FnMut($($name),*) -> std::result::Result<(), Err>,
            $($name: ContextBorrow<'a, Target = $name>,)*
        {
            fn execute(&mut self, context: &'a Context) -> Result<()> {
                (self)($($name::borrow(context)?), *)
                    .map_err(|e| Error::SystemError(self.name(), e.into()))
            }

            fn name(&self) -> SystemName {
                format!("System<{:?}>", type_name::<A>(),).into()
            }
        }
    };
}

tuple_impl!(A);
impl_for_tuples!(tuple_impl);

#[cfg(test)]
mod tests {
    use crate::{Borrow, Context, IntoData, SubWorld, System};
    use anyhow::ensure;
    use hecs::World;

    use super::Result;

    #[test]
    fn simple_system() {
        struct App {
            name: &'static str,
        }

        let mut app = App {
            name: "hecs-schedule",
        };

        let mut world = World::default();

        let a = world.spawn(("a", 3));
        let b = world.spawn(("b", 42));
        let c = world.spawn(("c", 8));

        let data = unsafe { (&mut world, &mut app).into_data() };

        let context = Context::new(&data);

        let mut count_system = |w: SubWorld<&i32>| assert_eq!(w.query::<&i32>().iter().count(), 3);
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

        let mut rename_system = |w: SubWorld<&mut String>, a: Borrow<App>| -> anyhow::Result<()> {
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
        assert!(name_query_system.execute(&context).is_err());
        name_check_system.execute(&context).unwrap();
        rename_system.execute(&context).unwrap();
        check_rename_system.execute(&context).unwrap();
    }
}
