use std::borrow::Cow;

use hecs::World;

use crate::{Context, Result};

/// System name alias
pub type SystemName = Cow<'static, str>;

/// Trait which defines any function or type that can operate on a world or
/// other context.
pub trait System {
    /// Executes the by borrowing from context
    fn execute(&mut self, context: &Context) -> Result<()>;
    /// Returns the system name. Used for debug purposes
    fn name(&self) -> SystemName;
}

/// Wrapper type for fallible systems returning any result
pub struct Fallible<F>(pub F);

impl<F, E> System for Fallible<F>
where
    E: Into<anyhow::Error>,
    F: FnMut(&mut World) -> std::result::Result<(), E>,
{
    fn execute(&mut self, context: &Context) -> Result<()> {
        let mut a = context.borrow::<&mut World>().unwrap();
        match (self.0)(&mut *a) {
            Ok(()) => Ok(()),
            Err(err) => Err(crate::Error::SystemError(self.name(), err.into())),
        }
    }

    fn name(&self) -> SystemName {
        "Fn(&mut World) -> ()".into()
    }
}

impl<F> System for F
where
    F: FnMut(&mut World),
{
    fn execute(&mut self, context: &Context) -> Result<()> {
        let mut a = context.borrow::<&mut World>().unwrap();
        (self)(&mut *a);
        Ok(())
    }

    fn name(&self) -> SystemName {
        "Fn(&mut World) -> ()".into()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Context, Fallible, IntoData, System};
    use anyhow::ensure;
    use hecs::World;

    #[test]
    fn simple_system() {
        let mut world = World::default();

        let a = world.spawn(("a", 3));
        let b = world.spawn(("b", 42));
        let c = world.spawn(("c", 8));

        let data = unsafe { (&mut world,).into_data() };

        let context = Context::new(&data);

        let mut system_a = |w: &mut World| assert_eq!(w.query::<&i32>().iter().count(), 3);
        let system_b = |w: &mut World| -> std::result::Result<(), hecs::ComponentError> {
            let name = w.get::<String>(a)?;
            eprintln!("Name: {:?}", *name);
            Ok(())
        };

        let system_c = |w: &mut World| -> anyhow::Result<()> {
            for (e, n) in [(a, "a"), (b, "b"), (c, "c")] {
                let name = w.get::<&str>(e)?;
                ensure!(*name == n, "Names did not match");
            }

            Ok(())
        };

        system_a.execute(&context).unwrap();
        assert!(Fallible(system_b).execute(&context).is_err());
        Fallible(system_c).execute(&context).unwrap();
    }
}
