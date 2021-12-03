use hecs::World;

pub trait System {
    // Executes the by borrowing from context
    fn execute(&self, world: &World);
}
