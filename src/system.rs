use hecs::World;

pub trait System {
    // Executes the system
    fn execute(&self, world: &World);
}
