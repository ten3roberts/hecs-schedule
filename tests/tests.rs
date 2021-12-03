use hecs::World;
use hecs_schedule::*;

#[test]
fn subworld() {
    let world = World::default();
    let subworld = SubWorld::<(&i32, &mut f32)>::new(&world);

    assert!(subworld.has::<&i32>());
    assert!(!subworld.has::<&mut i32>());
    assert!(subworld.has::<&f32>());
    assert!(subworld.has::<&mut f32>());

    assert!(subworld.has_all::<(&i32, &f32)>());
    assert!(!subworld.has_all::<(&mut i32, &f32)>());
    assert!(subworld.has_all::<(&mut f32, &i32)>());
    assert!(!subworld.has_all::<(&mut f32, &i32, &u32)>());
}
