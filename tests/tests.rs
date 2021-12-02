use hecs::World;
use hecs_schedule::*;

#[test]
fn subworld() {
    let world = World::default();
    let subworld = SubWorld::new::<&i32>(&world);

    assert!(subworld.has::<&i32>());
    assert!(!subworld.has::<&mut i32>());
}
