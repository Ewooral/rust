//! ECS part 2: systems are plain functions, a schedule runs them in order
//! every frame, and resources are the one-of-a-kind values they share.
//! Run: cargo run -p tauri_and_bevy --example ecs2_systems_and_resources

use tauri_and_bevy::ecs::{Commands, Schedule, World};

#[derive(Debug, Clone, Copy)]
struct Position(f64);
#[derive(Debug, Clone, Copy)]
struct Speed(f64);
#[derive(Debug, Clone)]
struct Name(&'static str);

/// A resource: the time step, the same for everyone.
struct DeltaTime(f64);
/// A resource: where the finish line is, and who has crossed it.
struct Race {
    finish: f64,
    finished: Vec<&'static str>,
}

/// Bevy: `fn movement(time: Res<Time>, mut q: Query<(&mut Position, &Speed)>)`
fn movement(world: &mut World) {
    let dt = world.resource::<DeltaTime>().0;
    world.each2_mut::<Position, Speed>(|_, position, speed| {
        position.0 += speed.0 * dt;
    });
}

/// Runners past the line leave the race. Despawning while we're looping
/// over Position would clash with the borrow, so we queue it in Commands.
fn finish_line(world: &mut World) {
    let finish = world.resource::<Race>().finish;
    let mut commands = Commands::default();
    let mut finishers = Vec::new();
    world.each::<Position>(|entity, position| {
        if position.0 >= finish {
            finishers.push(entity);
            commands.despawn(entity);
        }
    });
    for entity in finishers {
        let name = world.get::<Name>(entity).expect("every runner has a name");
        world.resource_mut::<Race>().finished.push(name.0);
    }
    commands.apply(world);
}

fn show(world: &mut World) {
    let mut line = String::new();
    world.each::<Position>(|entity, position| {
        let name = world.get::<Name>(entity).expect("named");
        line.push_str(&format!("{}={:.1}  ", name.0, position.0));
    });
    println!("  {line}");
}

fn main() {
    let mut world = World::new();
    world.insert_resource(DeltaTime(0.5));
    world.insert_resource(Race {
        finish: 10.0,
        finished: Vec::new(),
    });
    world.spawn((Name("tortoise"), Position(4.0), Speed(1.0)));
    world.spawn((Name("hare"), Position(0.0), Speed(3.0)));
    world.spawn((Name("snail"), Position(9.0), Speed(0.1)));

    // Bevy: app.add_systems(Update, (movement, finish_line, show).chain())
    let schedule = Schedule::new()
        .add("movement", movement)
        .add("finish_line", finish_line)
        .add("show", show);

    for frame in 1..=8 {
        println!("frame {frame}:");
        schedule.run(&mut world);
    }
    println!(
        "finished, in order: {:?}",
        world.resource::<Race>().finished
    );
}
