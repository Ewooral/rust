//! Solution to exercise 3: extend the critters game with new systems.
//!
//! Foxes now get hungry. Add a `Hunger` component and three systems,
//! written only with the lesson's `World` API, and run them after the
//! game's own systems:
//!
//! - `hunger_grows`: every fox's `Hunger` goes up by 1 each tick.
//!   (Hint: `each2_mut::<Hunger, Fox>` visits entities with both.)
//! - `eat`: if `Score.caught` went up since last tick, the foxes have eaten:
//!   their hunger goes back to 0. Remember the last score you saw in a
//!   resource of your own, `LastCaught`.
//! - `go_home`: a fox whose hunger reaches `STARVING` leaves the field:
//!   despawn it (with `Commands`, after the loop) and log
//!   `"tick 12: Rusty went home hungry"`.
//!
//! This is the ECS way to add a feature: new data (a component, a resource)
//! and new systems. Nothing in critters.rs had to change.
//!
//! Test: cargo test -p tauri_and_bevy --features solutions --bin sol03_hungry_foxes

use tauri_and_bevy::critters::{self, Fox, Log, Name, Score, Tick};
use tauri_and_bevy::ecs::{Commands, Schedule, World};

/// Ticks since the fox last ate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Hunger(u32);

/// The score `eat` saw last time.
#[derive(Debug, Default)]
struct LastCaught(u32);

/// At this hunger, a fox goes home.
const STARVING: u32 = 8;

fn hunger_grows(world: &mut World) {
    world.each2_mut::<Hunger, Fox>(|_, hunger, _| hunger.0 += 1);
}

fn eat(world: &mut World) {
    let caught = world.resource::<Score>().caught;
    let ate = caught > world.resource::<LastCaught>().0;
    world.resource_mut::<LastCaught>().0 = caught;
    if ate {
        world.each2_mut::<Hunger, Fox>(|_, hunger, _| hunger.0 = 0);
    }
}

fn go_home(world: &mut World) {
    let tick = world.resource::<Tick>().0;
    let mut commands = Commands::default();
    let mut gone = Vec::new();
    world.each2_mut::<Hunger, Fox>(|fox, hunger, _| {
        if hunger.0 >= STARVING {
            gone.push(fox);
            commands.despawn(fox);
        }
    });
    for fox in gone {
        let name = world
            .get::<Name>(fox)
            .map_or_else(|| "a fox".to_string(), |n| n.0);
        world
            .resource_mut::<Log>()
            .0
            .push(format!("tick {tick}: {name} went home hungry"));
    }
    commands.apply(world);
}

/// The game's systems, then ours.
fn schedule() -> Schedule {
    critters::schedule()
        .add("hunger_grows", hunger_grows)
        .add("eat", eat)
        .add("go_home", go_home)
}

/// A critters world where every fox starts with Hunger(0).
fn new_world(seed: u64) -> World {
    let mut world = critters::new_world(seed);
    world.insert_resource(LastCaught::default());
    for fox in world.with::<Fox>() {
        world.insert(fox, Hunger(0));
    }
    world
}

fn main() {
    let mut world = new_world(42);
    let schedule = schedule();
    for _ in 0..40 {
        schedule.run(&mut world);
    }
    print!("{}", critters::render(&world));
    for line in world.resource::<Log>().0.iter() {
        println!("  {line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A world with resources and one fox, but no rabbits.
    fn lonely_fox() -> (World, tauri_and_bevy::ecs::Entity) {
        let mut world = World::new();
        world.insert_resource(Tick(0));
        world.insert_resource(Score::default());
        world.insert_resource(Log::default());
        world.insert_resource(LastCaught::default());
        let fox = world.spawn((Fox, Hunger(0), Name("Rusty".into())));
        (world, fox)
    }

    #[test]
    fn hunger_grows_by_one_per_tick_for_foxes_only() {
        let (mut world, fox) = lonely_fox();
        let not_a_fox = world.spawn((Hunger(0),));
        hunger_grows(&mut world);
        hunger_grows(&mut world);
        assert_eq!(world.get::<Hunger>(fox), Some(Hunger(2)));
        assert_eq!(world.get::<Hunger>(not_a_fox), Some(Hunger(0)));
    }

    #[test]
    fn eating_resets_hunger() {
        let (mut world, fox) = lonely_fox();
        world.insert(fox, Hunger(4));
        eat(&mut world);
        assert_eq!(world.get::<Hunger>(fox), Some(Hunger(4)), "no catch yet");
        world.resource_mut::<Score>().caught += 1;
        eat(&mut world);
        assert_eq!(world.get::<Hunger>(fox), Some(Hunger(0)));
        world.insert(fox, Hunger(3));
        eat(&mut world);
        assert_eq!(
            world.get::<Hunger>(fox),
            Some(Hunger(3)),
            "only NEW catches count"
        );
    }

    #[test]
    fn a_starving_fox_goes_home() {
        let (mut world, fox) = lonely_fox();
        world.resource_mut::<Tick>().0 = 12;
        world.insert(fox, Hunger(STARVING - 1));
        go_home(&mut world);
        assert!(world.is_alive(fox));
        world.insert(fox, Hunger(STARVING));
        go_home(&mut world);
        assert!(!world.is_alive(fox));
        assert_eq!(
            world.resource::<Log>().0,
            ["tick 12: Rusty went home hungry"]
        );
    }

    #[test]
    fn in_the_full_game_no_living_fox_is_ever_starving_after_a_tick() {
        let mut world = new_world(42);
        let schedule = schedule();
        for _ in 0..60 {
            schedule.run(&mut world);
            world.each2_mut::<Hunger, Fox>(|_, hunger, _| assert!(hunger.0 < STARVING));
        }
    }
}
