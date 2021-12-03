use hecs::World;
use hecs_schedule::*;

#[test]
fn has() {
    let mut world = World::default();

    world.spawn((67_i32, 7.0_f32));

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

#[test]
fn query() {
    let mut world = World::default();

    world.spawn((67_i32, 7.0_f32));

    let subworld = SubWorld::<(&i32, &mut f32)>::new(&world);

    let mut query = subworld.query::<(&i32, &mut f32)>();
    query
        .iter()
        .for_each(|(e, val)| eprintln!("Entity {:?}: {:?}", e, val));

    assert!(subworld.try_query::<(&mut i32, &f32)>().is_err());
}
