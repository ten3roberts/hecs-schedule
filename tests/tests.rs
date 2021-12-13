use std::{thread::sleep, time::Duration};

use anyhow::{bail, ensure};
use hecs::{Query, World};
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

    let mut query = subworld.native_query();
    query
        .iter()
        .for_each(|(e, val)| eprintln!("Entity {:?}: {:?}", e, val));

    assert!(subworld.try_query::<(&mut i32, &f32)>().is_err());
    let val = subworld.try_get::<i32>(entity).unwrap();
    assert_eq!(*val, 42);
}

#[test]
fn custom_query() {
    let mut world = World::default();

    #[derive(Query, Debug)]
    struct Foo<'a> {
        _a: &'a i32,
        _b: &'a mut f32,
    }

    world.spawn((67_i32, 7.0_f32));
    let entity = world.spawn((42_i32, 3.1415_f32));

    let subworld = SubWorldRef::<(Foo, &&'static str)>::new(&world);

    assert!(subworld.has_all::<(&i32, &f32)>());
    assert!(!subworld.has_all::<(&mut i32, &f32)>());
    assert!(subworld.has_all::<(&mut f32, &i32)>());
    assert!(subworld.has_all::<(&&'static str, &i32)>());
    assert!(!subworld.has_all::<(&mut &'static str, &i32)>());
    assert!(!subworld.has_all::<(&mut f32, &i32, &u32)>());

    let column = subworld.try_get_column::<i32>().unwrap();
    let mut query = subworld.try_query_one::<&i32>(entity).unwrap();
    let val = query.get().unwrap();
    assert_eq!(*val, 42);

    let mut query = subworld.query::<Foo>();
    query
        .iter()
        .for_each(|(e, val)| eprintln!("Entity {:?}: {:?}", e, val));

    assert!(subworld.try_query::<(&mut i32, &f32)>().is_err());
    let val = column.get(entity).unwrap();
    assert_eq!(*val, 42);
}

#[test]
#[should_panic]
fn fail_query() {
    let mut world = World::default();

    let entity = world.spawn((42_i32, 3.1415_f32));

    let subworld = SubWorldRef::<(&i32, &f32)>::new(&world);

    let val = subworld.try_get::<u64>(entity).unwrap();
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
    let b = world.reserve_entity();

    let mut schedule = Schedule::builder();

    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    struct Foo {
        val: i32,
    }

    let mut foo = Foo { val: 42 };

    let system = move |w: SubWorld<&i32>| -> anyhow::Result<()> {
        ensure!(*w.try_get::<i32>(a)? == 789, "Entity did not match");

        Ok(())
    };

    let spawn_system = move |mut cmd: Write<CommandBuffer>| {
        cmd.insert(b, ("b", 8));
    };

    schedule.add_system(spawn_system);
    schedule.add_system(system);
    schedule.flush();

    schedule.add_system(|mut val: Write<Foo>| {
        val.val = 56;
    });

    schedule.add_system(move |mut cmd: Write<CommandBuffer>| {
        cmd.insert_one(a, "Hello, World!");
    });

    schedule.add_system(
        move |w: SubWorld<(&&'static str, &i32)>| -> anyhow::Result<()> {
            let mut query = w.try_query_one::<(&&'static str, &i32)>(b)?;
            let (name, val) = query.get()?;

            ensure!(*name == "b" && *val == 8, "Entity does not match");

            Ok(())
        },
    );

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
        sleep(Duration::from_millis(200));
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

    eprintln!("{}", schedule.batch_info());

    schedule
        .execute((&mut val, &mut other_val))
        .map_err(|e| eprintln!("Error {}", e))
        .expect("Failed to execute schedule");
}

#[test]
fn execute_par_rw() {
    #[derive(Debug, PartialEq, Eq)]
    struct A(i32);
    #[derive(Debug, PartialEq, Eq)]
    struct B(i32);
    #[derive(Debug, PartialEq, Eq)]
    struct C(i32);

    let mut a = A(5);
    let mut b = B(7);
    let mut c = C(8);

    let outer = "Foo";
    let outer2 = "Bar";

    let mut world = World::default();

    fn system1(a: Read<A>, b: Read<B>, c: Read<C>) {
        assert_eq!(*a, A(5));
        assert_eq!(*b, B(7));
        assert_eq!(*c, C(8));
    }

    fn system2(mut a: Write<A>, outer: &str) {
        sleep(Duration::from_millis(100));
        *a = A(11);
        assert_eq!(outer, "Foo");
    }

    fn system3(a: Read<A>, outer: &str) {
        assert_eq!(*a, A(11));
        assert_eq!(outer, "Bar");
    }

    let mut schedule = Schedule::builder()
        .add_system(
            |_: SubWorld<(&A, &B)>, a: Read<_>, b: Read<_>, c: Read<_>| {
                system1(a, b, c);
            },
        )
        .add_system(move |_: SubWorld<&i32>, a: Write<_>| system2(a, outer))
        .add_system(move |_: Read<C>, a: Read<_>| system3(a, outer2))
        .build();

    eprintln!("Batches: {}", schedule.batch_info());

    schedule
        .execute((&mut world, &mut a, &mut b, &mut c))
        .unwrap();
}
