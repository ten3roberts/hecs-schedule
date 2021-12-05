use std::{thread::sleep, time::Duration};

use anyhow::{bail, ensure};
use hecs::World;
use hecs_schedule::*;

#[test]
fn has() {
    let mut world = World::default();

    world.spawn((67_i32, 7.0_f32));

    let subworld = SubWorldRef::<(&i32, &mut f32)>::new(&world);

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
    let entity = world.spawn((42_i32, 3.1415_f32));

    let subworld = SubWorldRef::<(&i32, &mut f32)>::new(&world);

    let mut query = subworld.query::<(&i32, &mut f32)>();
    query
        .iter()
        .for_each(|(e, val)| eprintln!("Entity {:?}: {:?}", e, val));

    assert!(subworld.try_query::<(&mut i32, &f32)>().is_err());
    let val = subworld.get::<i32>(entity).unwrap();
    assert_eq!(*val, 42);
}

#[test]
#[should_panic]
fn fail_query() {
    let mut world = World::default();

    let entity = world.spawn((42_i32, 3.1415_f32));

    let subworld = SubWorldRef::<(&i32, &f32)>::new(&world);

    let val = subworld.get::<u64>(entity).unwrap();
    assert_eq!(*val, 42);
}

#[test]
fn commandbuffer() {
    let mut world = World::default();
    let e = world.reserve_entity();

    let mut cmds = CommandBuffer::default();

    cmds.spawn((42_i32, 7.0_f32));
    cmds.insert(e, (89_usize, 42_i32, String::from("Foo")));

    cmds.remove_one::<usize>(e);

    cmds.execute(&mut world);

    assert!(world
        .query::<(&i32, &f32)>()
        .iter()
        .map(|(_, val)| val)
        .eq([(&42, &7.0)]))
}

#[test]
fn test_schedule() {
    let mut world = World::default();

    let a = world.spawn((789,));

    let mut schedule = Schedule::builder();

    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    struct Foo {
        val: i32,
    }

    let mut foo = Foo { val: 42 };

    let system = move |w: SubWorld<&i32>| -> anyhow::Result<()> {
        ensure!(*w.get::<i32>(a)? == 789, "Entity did not match");

        Ok(())
    };

    schedule.add_system(system);

    schedule.add_system(|mut val: Write<Foo>| {
        val.val = 56;
    });

    let mut schedule = schedule.build();
    schedule.execute_seq((&mut world, &mut foo)).unwrap();

    assert_eq!(foo, Foo { val: 56 });
}

#[test]
#[should_panic]
fn schedule_fail() {
    let mut schedule = Schedule::builder()
        .add_system(|| -> anyhow::Result<()> { bail!("Dummy Error") })
        .build();

    schedule.execute_seq(()).unwrap();
}

#[test]
fn execute_par() {
    let mut val = 3;
    let mut other_val = 3.0;
    let observe_before = |val: Read<i32>| {
        sleep(Duration::from_millis(100));
        assert_eq!(*val, 3)
    };

    // Should execute at the same time as ^
    let observe_other = |val: Read<f64>| {
        sleep(Duration::from_millis(100));
        assert_eq!(*val, 3.0);
    };

    let mutate = |mut val: Write<i32>| {
        sleep(Duration::from_millis(20));
        *val = 5;
    };

    let observe_after = |val: Read<i32>| {
        assert_eq!(*val, 5);
    };

    let mut other_schedule = Schedule::builder();
    other_schedule.add_system(observe_other).add_system(mutate);

    let mut schedule = Schedule::builder()
        .add_system(observe_before)
        .append(&mut other_schedule)
        .add_system(observe_after)
        .build();

    schedule
        .execute((&mut val, &mut other_val))
        .map_err(|e| eprintln!("Error {}", e))
        .expect("Failed to execute schedule");
}
