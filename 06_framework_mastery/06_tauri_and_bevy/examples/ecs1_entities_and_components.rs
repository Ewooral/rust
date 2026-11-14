//! ECS part 1: entities are ids, components are data, stored by TYPE.
//! Run: cargo run -p tauri_and_bevy --example ecs1_entities_and_components

use tauri_and_bevy::ecs::World;

#[derive(Debug, Clone)]
struct Name(&'static str);
#[derive(Debug, Clone)]
struct Health(u32);
#[derive(Debug, Clone)]
struct Flying;

fn main() {
    let mut world = World::new();

    // An object-oriented game might write `struct Dragon { name, health, wings }`.
    // In an ECS, a dragon is just an id with whichever components it has.
    let dragon = world.spawn((Name("Smaug"), Health(500), Flying));
    let knight = world.spawn((Name("Bilbo"), Health(30)));
    let rock = world.spawn((Name("a rock"),));

    for entity in world.entities() {
        println!(
            "{entity:?}: name {:?}, health {:?}, flying {}",
            world.get::<Name>(entity).map(|n| n.0),
            world.get::<Health>(entity).map(|h| h.0),
            world.has::<Flying>(entity),
        );
    }

    // "Which entities can fly?" is a question about one storage, not a
    // search through every object.
    println!("\nflying: {:?}", world.with::<Flying>());
    println!("have health: {:?}", world.with::<Health>());

    // Adding a component changes what an entity IS, at run time.
    world.insert(knight, Flying); // a giant eagle picks him up
    println!("flying now: {:?}", world.with::<Flying>());

    // Despawn, then spawn again: the index is reused, the generation isn't.
    world.despawn(rock);
    let pebble = world.spawn((Name("a pebble"),));
    println!(
        "\nrock was {rock:?}, pebble is {pebble:?}: same index {}, generations {} and {}",
        pebble.index(),
        rock.generation(),
        pebble.generation()
    );
    println!(
        "the old rock id is alive: {}; it finds a Name: {:?}",
        world.is_alive(rock),
        world.get::<Name>(rock)
    );
    let _ = dragon;
}
