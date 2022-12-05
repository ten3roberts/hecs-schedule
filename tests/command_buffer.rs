use anyhow::ensure;
use hecs::World;
use hecs_schedule::{CommandBuffer, GenericWorld, Schedule, SubWorld, Write};

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

    let mut value = Foo { val: 42 };

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
    schedule.execute_seq((&mut world, &mut value)).unwrap();

    assert_eq!(value, Foo { val: 56 });
}
