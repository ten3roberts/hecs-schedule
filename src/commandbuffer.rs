use hecs::{
    Bundle, CommandBuffer as CommandBufferInternal, Component, DynamicBundle, Entity, World,
};

#[derive(Default)]
/// Extends the built in [hecs::CommandBuffer].
///
/// Allows for deferred modifications to the world, spawn, insert, remove,
/// despawn, or custom closures.
///
/// It is possible to insert a commandbuffer into another commandbuffer.
pub struct CommandBuffer {
    /// Use the already existing hecs::CommmandBuffer
    components: CommandBufferInternal,
    despawns: Vec<Entity>,
    writes: Vec<Box<dyn FnOnce(&mut World) + Send + Sync>>,
}

impl CommandBuffer {
    /// Creates a new empty commandbuffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts components into an already existing or reserved entity
    pub fn insert(&mut self, entity: Entity, components: impl DynamicBundle) {
        self.components.insert(entity, components)
    }

    /// Inserts a single component into an already existing or reserved entity
    pub fn insert_one(&mut self, entity: Entity, component: impl Component) {
        self.components.insert(entity, (component,))
    }

    /// Spawns a new entity with components.
    /// If the entity ID is desired, consider reserving an entity and then inserting
    pub fn spawn(&mut self, components: impl DynamicBundle) {
        self.components.spawn(components)
    }

    /// Despawn an entity from the world
    pub fn despawn(&mut self, entity: Entity) {
        self.despawns.push(entity)
    }

    /// Remove components from entity
    pub fn remove<C: Component + Bundle>(&mut self, entity: Entity) {
        self.writes.push(Box::new(move |w| {
            w.remove::<C>(entity).unwrap();
        }))
    }

    /// Remove a single component from the world
    pub fn remove_one<C: Component>(&mut self, entity: Entity) {
        self.writes.push(Box::new(move |w| {
            w.remove_one::<C>(entity).unwrap();
        }))
    }

    /// Applies the recorded commands on the world
    pub fn execute(&mut self, world: &mut World) {
        self.components.run_on(world);

        self.writes.drain(..).for_each(|cmd| (cmd)(world));

        self.despawns
            .drain(..)
            .for_each(|e| world.despawn(e).expect("Failed to despawn entity"));
    }

    /// Nest a commandbuffer
    pub fn append(&mut self, mut other: Self) {
        self.write(move |w| other.execute(w))
    }

    /// Record a custom command modifying the world
    pub fn write(&mut self, cmd: impl FnOnce(&mut World) + Component) {
        self.writes.push(Box::new(cmd))
    }

    /// Drop all recorded commands
    pub fn clear(&mut self) {
        self.despawns.clear();
        self.writes.clear();
        self.components.clear();
    }
}

