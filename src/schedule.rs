use std::{
    any::TypeId,
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use smallvec::SmallVec;

use crate::{Access, Borrows, Context, IntoData, Result, System};

#[derive(Default)]
/// Represents a unit of work with compatible borrows.
pub struct Batch {
    systems: SmallVec<[DynamicSystem; 8]>,
}

impl Batch {
    pub fn push<Args, Ret, S>(&mut self, system: S)
    where
        S: 'static + System<Args, Ret>,
    {
        self.systems.push(DynamicSystem::new(system))
    }
}

impl Deref for Batch {
    type Target = [DynamicSystem];

    fn deref(&self) -> &Self::Target {
        &self.systems
    }
}

impl DerefMut for Batch {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.systems
    }
}

// Type erased boxed system
#[doc(hidden)]
pub struct DynamicSystem {
    func: Box<dyn FnMut(&Context) -> Result<()>>,
}

#[doc(hidden)]
impl DynamicSystem {
    fn new<S, Args, Ret>(mut system: S) -> Self
    where
        S: 'static + System<Args, Ret>,
    {
        Self {
            func: Box::new(move |context| system.execute(context)),
        }
    }

    fn execute(&mut self, context: &Context) -> Result<()> {
        (self.func)(context)
    }
}

/// A shedule represents a collections of system which will run with effects in
/// a determined order.
pub struct Schedule {
    batches: Vec<Batch>,
}

impl Schedule {
    pub fn new(batches: Vec<Batch>) -> Self {
        Self { batches }
    }

    pub fn builder() -> ScheduleBuilder {
        ScheduleBuilder::default()
    }

    /// Executes the systems inside the schedule  using the provided data, which
    /// is a tuple of mutable references. Returns Err if any system fails
    pub fn execute<D: IntoData>(&mut self, data: D) -> Result<()> {
        let data = unsafe { data.into_data() };

        let context = Context::new(&data);

        self.batches.iter_mut().try_for_each(|batch| {
            batch
                .iter_mut()
                .try_for_each(|system| system.execute(&context))
        })
    }
}

#[derive(Default)]
pub struct ScheduleBuilder {
    batches: Vec<Batch>,
    current_batch: Batch,
    current_borrows: HashMap<TypeId, Access>,
}

impl ScheduleBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_system<Args, Ret, S>(&mut self, system: S) -> &mut Self
    where
        S: 'static + System<Args, Ret>,
    {
        // Check borrow
        let borrows = S::borrows();

        if !self.check_compatible(&borrows) {
            // Push and create a new batch
            self.push_batch()
        }

        self.add_borrows(borrows);
        self.current_batch.push(system);

        self
    }

    fn push_batch(&mut self) {
        let batch = std::mem::replace(&mut self.current_batch, Batch::default());

        self.batches.push(batch);

        self.current_borrows.clear()
    }

    fn add_borrows(&mut self, borrows: Borrows) {
        self.current_borrows
            .extend(borrows.into_iter().map(|val| (val.id(), val)))
    }

    /// Returns true if no borrows conflict with the current ones
    fn check_compatible(&self, borrows: &Borrows) -> bool {
        for borrow in borrows {
            // Type is already borrowd
            if let Some(curr) = self.current_borrows.get(&borrow.id()) {
                // Already exclusively borroed or new borrow is exlcusive
                return !curr.exclusive() && (borrow.exclusive());
            }
        }

        return true;
    }

    /// Moves the current batches into a schedule
    pub fn build(&mut self) -> Schedule {
        // Push the current batch
        self.push_batch();

        let builder = std::mem::replace(self, ScheduleBuilder::default());

        Schedule::new(builder.batches)
    }
}
