use std::{
    any::TypeId,
    collections::HashMap,
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

use hecs::World;
use smallvec::SmallVec;

#[cfg(feature = "parallel")]
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    borrow::{Borrows, MaybeWrite},
    Access, CommandBuffer, Context, IntoData, Result, System, SystemName, Write,
};

#[derive(Default, Debug, Clone)]
/// Holds information regarding batches
pub struct BatchInfo<'a> {
    batches: &'a [Batch],
}

impl<'a> Display for BatchInfo<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Batches: \n")?;
        for batch in self.batches {
            for system in &batch.systems {
                write!(f, " - {}\n", system.name())?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

#[derive(Default)]
/// Represents a unit of work with compatible borrows.
pub struct Batch {
    systems: SmallVec<[DynamicSystem; 8]>,
    has_flush: bool,
}

impl Debug for Batch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();

        for system in self.systems() {
            list.entry(&system.name());
        }

        list.finish()
    }
}

impl Batch {
    fn push(&mut self, system: DynamicSystem) {
        self.systems.push(system)
    }

    /// Get a reference to the batch's systems.
    pub fn systems(&self) -> &SmallVec<[DynamicSystem; 8]> {
        &self.systems
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
    func: Box<dyn FnMut(&Context) -> Result<()> + Send>,
    name: SystemName,
    borrows: Borrows,
}

#[doc(hidden)]
impl DynamicSystem {
    fn new<S, Args, Ret>(mut system: S) -> Self
    where
        S: 'static + System<Args, Ret> + Send,
    {
        let borrows = S::borrows();
        let name = system.name();
        Self {
            func: Box::new(move |context| system.execute(context)),
            name,
            borrows,
        }
    }

    fn execute(&mut self, context: &Context) -> Result<()> {
        (self.func)(context)
    }

    /// Get a reference to the dynamic system's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
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

    /// Returns information of how the schedule was split into batches
    pub fn batch_info(&self) -> BatchInfo {
        BatchInfo {
            batches: &self.batches,
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
        S: 'static + System<Args, Ret> + Send,
    {
        self.add_internal(DynamicSystem::new(system));
        self
    }

    fn add_internal(&mut self, system: DynamicSystem) {
        // Check borrow
        let borrows = &system.borrows;

        if !self.check_compatible(borrows) {
            // Push and create a new batch
            self.barrier();
        }

        self.add_borrows(borrows);
        self.current_batch.push(system);
    }

    /// Append all system from `other` into self, leaving `other` empty.
    /// This allows constructing smaller schedules in different modules and then
    /// joining them together. Work will be paralellized between the two
    /// schedules.
    pub fn append(&mut self, other: &mut ScheduleBuilder) -> &mut Self {
        other.barrier();

        other.batches.drain(..).for_each(|mut batch| {
            batch
                .systems
                .drain(..)
                .for_each(|system| self.add_internal(system))
        });

        self
    }

    /// Inserts a barrier that will divide the schedule pararell execution in
    /// two dependant halves.
    ///
    /// Usually this is not required, as the borrows of the system automatically
    /// creates dependencies, but sometimes a manual dependency is needed for things
    /// such as interior mutability or channels.
    pub fn barrier(&mut self) -> &mut Self {
        let batch = std::mem::take(&mut self.current_batch);

        self.batches.push(batch);

        self.current_borrows.clear();

        self
    }

    /// Flush the commandbuffer and apply the commands to the world
    pub fn flush(&mut self) -> &mut Self {
        self.current_batch.has_flush = true;
        self.add_system(flush_system)
    }

    fn add_borrows(&mut self, borrows: &Borrows) {
        self.current_borrows
            .extend(borrows.into_iter().map(|val| (val.id(), val.clone())))
    }

    /// Returns true if no borrows conflict with the current ones
    fn check_compatible(&self, borrows: &Borrows) -> bool {
        for borrow in borrows {
            // Type is already borrowed
            if let Some(curr) = self.current_borrows.get(&borrow.id()) {
                // Already exclusively borrowed or new borrow is exlcusive
                if curr.exclusive() || borrow.exclusive() {
                    return false;
                }
            }
        }

        true
    }

    /// FLushes the commandbuffer and builds the schedule.
    pub fn build(&mut self) -> Schedule {
        self.flush();
        // Push the current batch
        self.barrier();

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
