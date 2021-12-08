use std::{
    any::TypeId,
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use hecs::World;
use smallvec::SmallVec;

#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    borrow::{Borrows, MaybeWrite},
    Access, CommandBuffer, Context, IntoData, Result, System, Write,
};

#[derive(Default)]
/// Represents a unit of work with compatible borrows.
pub struct Batch {
    systems: SmallVec<[DynamicSystem; 8]>,
}

impl Batch {
    fn push(&mut self, system: DynamicSystem) {
        self.systems.push(system)
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
    func: Box<dyn FnMut(&Context) -> Result<()> + Send + Sync>,
    borrows: Borrows,
}

#[doc(hidden)]
impl DynamicSystem {
    fn new<S, Args, Ret>(mut system: S) -> Self
    where
        S: 'static + System<Args, Ret> + Send + Sync,
    {
        let borrows = S::borrows();
        Self {
            func: Box::new(move |context| system.execute(context)),
            borrows,
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
    cmd: CommandBuffer,
}

impl Schedule {
    /// Creates a new schedule from provided batches.
    pub fn new(batches: Vec<Batch>) -> Self {
        Self {
            batches,
            cmd: Default::default(),
        }
    }

    /// Creates a new [ScheduleBuilder]
    pub fn builder() -> ScheduleBuilder {
        ScheduleBuilder::default()
    }

    /// Executes the systems inside the schedule sequentially using the provided data, which
    /// is a tuple of mutable references. Returns Err if any system fails.
    ///
    /// A commandbuffer is always available and will be flushed at the end.
    pub fn execute_seq<D: IntoData<CommandBuffer>>(&mut self, data: D) -> Result<()> {
        let data = unsafe { data.into_data(&mut self.cmd) };

        let context = Context::new(&data);

        self.batches.iter_mut().try_for_each(|batch| {
            batch
                .iter_mut()
                .try_for_each(|system| system.execute(&context))
        })
    }

    #[cfg(feature = "parallel")]
    /// Executes the systems inside the schedule ina parallel using the provided data, which
    /// is a tuple of mutable references. Returns Err if any system fails
    ///
    /// A commandbuffer is always available and will be flushed at the end.
    pub fn execute<D: IntoData<CommandBuffer> + Send + Sync>(&mut self, data: D) -> Result<()> {
        let data = unsafe { data.into_data(&mut self.cmd) };

        let context = Context::new(&data);

        self.batches.iter_mut().try_for_each(|batch| {
            batch
                .par_iter_mut()
                .try_for_each(|system| system.execute(&context))
        })
    }

    /// Get a reference to the schedule's cmd.
    pub fn cmd(&self) -> &CommandBuffer {
        &self.cmd
    }

    /// Get a mutable reference to the schedule's cmd.
    pub fn cmd_mut(&mut self) -> &mut CommandBuffer {
        &mut self.cmd
    }
}

#[derive(Default)]
/// Builder for incrementally constructing a schedule.
pub struct ScheduleBuilder {
    batches: Vec<Batch>,
    current_batch: Batch,
    current_borrows: HashMap<TypeId, Access>,
}

impl ScheduleBuilder {
    /// Creates a new [ScheduleBuilder]
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a system to the builder
    pub fn add_system<Args, Ret, S>(&mut self, system: S) -> &mut Self
    where
        S: 'static + System<Args, Ret> + Send + Sync,
    {
        self.add_internal(DynamicSystem::new(system));
        self
    }

    fn add_internal(&mut self, system: DynamicSystem) {
        // Check borrow
        let borrows = &system.borrows;

        if !self.check_compatible(borrows) {
            // Push and create a new batch
            self.finalize_batch();
        }

        self.add_borrows(borrows);
        self.current_batch.push(system);
    }

    /// Append all system from `other` into self, leaving `other` empty.
    /// This allows constructing smaller schedules in different modules and then
    /// joining them together. Work will be paralellized between the two
    /// schedules.
    pub fn append(&mut self, other: &mut ScheduleBuilder) -> &mut Self {
        other.finalize_batch();

        other.batches.drain(..).for_each(|mut batch| {
            batch
                .systems
                .drain(..)
                .for_each(|system| self.add_internal(system))
        });

        self
    }

    /// Flush the commandbuffer and apply the commands to the world
    pub fn flush(&mut self) -> &mut Self {
        self.add_system(flush_system)
    }

    fn finalize_batch(&mut self) {
        let batch = std::mem::take(&mut self.current_batch);

        self.batches.push(batch);

        self.current_borrows.clear()
    }

    fn add_borrows(&mut self, borrows: &Borrows) {
        self.current_borrows
            .extend(borrows.into_iter().map(|val| (val.id(), val.to_owned())))
    }

    /// Returns true if no borrows conflict with the current ones
    fn check_compatible(&self, borrows: &Borrows) -> bool {
        for borrow in borrows {
            // Type is already borrowd&
            if let Some(curr) = self.current_borrows.get(&borrow.id()) {
                // Already exclusively borroed or new borrow is exlcusive
                return !curr.exclusive() && !(borrow.exclusive());
            }
        }

        true
    }

    /// FLushes the commandbuffer and builds the schedule.
    pub fn build(&mut self) -> Schedule {
        self.flush();
        // Push the current batch
        self.finalize_batch();

        let builder = std::mem::take(self);

        Schedule::new(builder.batches)
    }
}

// Flushes the commandbuffer
fn flush_system(mut world: MaybeWrite<World>, mut cmd: Write<CommandBuffer>) -> Result<()> {
    if let Some(world) = world.option_mut() {
        cmd.execute(world);
    }
    Ok(())
}
