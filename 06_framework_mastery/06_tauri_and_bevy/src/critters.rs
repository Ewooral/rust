//! A text-mode simulation built on the tiny ECS: rabbits hop around a
//! field, new ones appear now and then, and a fox chases the nearest one.
//! Everything that happens is a system; everything that exists is an
//! entity with components.

use crate::ecs::{Commands, Entity, Schedule, World};

// ---------- components: plain data, no behaviour ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Velocity {
    pub dx: i32,
    pub dy: i32,
}

/// A marker component: no data, it just says "this is a rabbit".
#[derive(Debug, Clone, Copy)]
pub struct Rabbit;

#[derive(Debug, Clone, Copy)]
pub struct Fox;

/// What the entity looks like on screen.
#[derive(Debug, Clone, Copy)]
pub struct Glyph(pub char);

#[derive(Debug, Clone)]
pub struct Name(pub String);

// ---------- resources: one of each in the whole world ----------

#[derive(Debug, Clone, Copy)]
pub struct Arena {
    pub width: i32,
    pub height: i32,
}

/// How many frames have run.
#[derive(Debug, Default)]
pub struct Tick(pub u64);

#[derive(Debug, Default)]
pub struct Score {
    pub caught: u32,
}

/// What happened, for printing.
#[derive(Debug, Default)]
pub struct Log(pub Vec<String>);

/// A tiny deterministic random number generator, so every run (and every
/// test) sees the same field.
#[derive(Debug)]
pub struct Rng(pub u64);

impl Rng {
    pub fn below(&mut self, n: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) % n
    }
}

/// The most rabbits on the field at once.
pub const MAX_RABBITS: usize = 6;
/// A new rabbit appears every this many ticks.
pub const SPAWN_EVERY: u64 = 4;

/// A world with an arena, a fox, three rabbits, and the resources.
pub fn new_world(seed: u64) -> World {
    let mut world = World::new();
    world.insert_resource(Arena {
        width: 24,
        height: 8,
    });
    world.insert_resource(Tick::default());
    world.insert_resource(Score::default());
    world.insert_resource(Log::default());
    world.insert_resource(Rng(seed));

    world.spawn((
        Position { x: 0, y: 0 },
        Fox,
        Glyph('F'),
        Name("Rusty".into()),
    ));
    for _ in 0..3 {
        spawn_rabbit(&mut world);
    }
    world
}

/// The systems, in the order they run each tick.
pub fn schedule() -> Schedule {
    Schedule::new()
        .add("advance_tick", advance_tick)
        .add("spawn_rabbits", spawn_rabbits)
        .add("move_rabbits", move_rabbits)
        .add("chase", chase)
        .add("catch", catch)
}

fn spawn_rabbit(world: &mut World) -> Entity {
    let arena = *world.resource::<Arena>();
    let (x, y, dx, dy) = {
        let mut rng = world.resource_mut::<Rng>();
        let x = rng.below(arena.width as u64) as i32;
        let y = rng.below(arena.height as u64) as i32;
        let dx = [-1, 1][rng.below(2) as usize];
        let dy = [-1, 0, 1][rng.below(3) as usize];
        (x, y, dx, dy)
    }; // the RefMut is dropped here, before we spawn
    world.spawn((Position { x, y }, Velocity { dx, dy }, Rabbit, Glyph('r')))
}

// ---------- systems: all the behaviour ----------

/// Counts frames.
pub fn advance_tick(world: &mut World) {
    world.resource_mut::<Tick>().0 += 1;
}

/// Every few ticks, a new rabbit (if there's room).
pub fn spawn_rabbits(world: &mut World) {
    let tick = world.resource::<Tick>().0;
    if tick.is_multiple_of(SPAWN_EVERY) && world.with::<Rabbit>().len() < MAX_RABBITS {
        let rabbit = spawn_rabbit(world);
        let p = world.get::<Position>(rabbit).expect("just spawned");
        world.resource_mut::<Log>().0.push(format!(
            "tick {tick}: a rabbit appears at ({}, {})",
            p.x, p.y
        ));
    }
}

/// Everything with a Velocity moves, bouncing off the edges.
pub fn move_rabbits(world: &mut World) {
    let arena = *world.resource::<Arena>();
    // Velocity can change (a bounce), Position always does: so this query
    // is (&mut Position, &mut Velocity)... which our tiny ECS can't express.
    // Instead: read velocities, write positions, then write the bounces.
    let mut bounces = Vec::new();
    world.each2_mut::<Position, Velocity>(|entity, p, v| {
        let (mut dx, mut dy) = (v.dx, v.dy);
        if !(0..arena.width).contains(&(p.x + dx)) {
            dx = -dx;
        }
        if !(0..arena.height).contains(&(p.y + dy)) {
            dy = -dy;
        }
        p.x += dx;
        p.y += dy;
        if (dx, dy) != (v.dx, v.dy) {
            bounces.push((entity, Velocity { dx, dy }));
        }
    });
    for (entity, velocity) in bounces {
        world.insert(entity, velocity);
    }
}

/// The fox steps one square towards the nearest rabbit.
pub fn chase(world: &mut World) {
    // Collect the rabbits' positions first: we can't read Position for the
    // rabbits while writing Position for the fox (one storage, one borrow).
    let rabbits: Vec<Position> = world
        .with::<Rabbit>()
        .into_iter()
        .filter_map(|r| world.get::<Position>(r))
        .collect();
    for fox in world.with::<Fox>() {
        let Some(mut p) = world.get::<Position>(fox) else {
            continue;
        };
        let nearest = rabbits
            .iter()
            .min_by_key(|r| (r.x - p.x).abs() + (r.y - p.y).abs());
        if let Some(target) = nearest {
            p.x += (target.x - p.x).signum();
            p.y += (target.y - p.y).signum();
            world.insert(fox, p);
        }
    }
}

/// A rabbit on the fox's square (or next to it) is caught.
pub fn catch(world: &mut World) {
    let tick = world.resource::<Tick>().0;
    let mut commands = Commands::default();
    let mut caught = Vec::new();
    for fox in world.with::<Fox>() {
        let (Some(fp), Some(name)) = (world.get::<Position>(fox), world.get::<Name>(fox)) else {
            continue;
        };
        for rabbit in world.with::<Rabbit>() {
            let Some(rp) = world.get::<Position>(rabbit) else {
                continue;
            };
            if (rp.x - fp.x).abs() <= 1 && (rp.y - fp.y).abs() <= 1 && !caught.contains(&rabbit) {
                caught.push(rabbit);
                commands.despawn(rabbit);
                world.resource_mut::<Log>().0.push(format!(
                    "tick {tick}: {} caught a rabbit at ({}, {})",
                    name.0, rp.x, rp.y
                ));
            }
        }
    }
    world.resource_mut::<Score>().caught += caught.len() as u32;
    commands.apply(world);
}

/// Draws the field as text. Not a system: it only reads the world.
pub fn render(world: &World) -> String {
    let arena = *world.resource::<Arena>();
    let mut grid = vec![vec!['.'; arena.width as usize]; arena.height as usize];
    // Rabbits first, then the fox, so the fox is drawn on top.
    let mut drawables: Vec<(Entity, Position, char)> = world
        .entities()
        .into_iter()
        .filter_map(|e| Some((e, world.get::<Position>(e)?, world.get::<Glyph>(e)?.0)))
        .collect();
    drawables.sort_by_key(|&(e, _, _)| world.has::<Fox>(e));
    for (_, p, glyph) in drawables {
        if (0..arena.width).contains(&p.x) && (0..arena.height).contains(&p.y) {
            grid[p.y as usize][p.x as usize] = glyph;
        }
    }
    let rows: Vec<String> = grid
        .into_iter()
        .map(|row| row.into_iter().collect())
        .collect();
    format!(
        "tick {:>3}  rabbits {}  caught {}\n{}\n",
        world.resource::<Tick>().0,
        world.with::<Rabbit>().len(),
        world.resource::<Score>().caught,
        rows.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_world_has_a_fox_and_three_rabbits() {
        let world = new_world(7);
        assert_eq!(world.with::<Fox>().len(), 1);
        assert_eq!(world.with::<Rabbit>().len(), 3);
        assert_eq!(world.resource::<Tick>().0, 0);
    }

    #[test]
    fn rabbits_bounce_off_the_edges() {
        let mut world = new_world(1);
        let rabbit = world.spawn((Position { x: 0, y: 0 }, Velocity { dx: -1, dy: -1 }, Rabbit));
        move_rabbits(&mut world);
        assert_eq!(world.get::<Position>(rabbit), Some(Position { x: 1, y: 1 }));
        assert_eq!(
            world.get::<Velocity>(rabbit),
            Some(Velocity { dx: 1, dy: 1 })
        );
    }

    #[test]
    fn rabbits_stay_inside_the_arena_forever() {
        let mut world = new_world(3);
        let schedule = schedule();
        for _ in 0..200 {
            schedule.run(&mut world);
            let arena = *world.resource::<Arena>();
            world.each::<Position>(|_, p| {
                assert!((0..arena.width).contains(&p.x) && (0..arena.height).contains(&p.y));
            });
        }
    }

    #[test]
    fn the_fox_moves_towards_the_nearest_rabbit() {
        let mut world = World::new();
        let fox = world.spawn((Position { x: 0, y: 0 }, Fox));
        world.spawn((Position { x: 5, y: 3 }, Rabbit));
        world.spawn((Position { x: 20, y: 0 }, Rabbit));
        chase(&mut world);
        assert_eq!(world.get::<Position>(fox), Some(Position { x: 1, y: 1 }));
    }

    #[test]
    fn catching_despawns_the_rabbit_and_scores() {
        let mut world = World::new();
        world.insert_resource(Tick(9));
        world.insert_resource(Score::default());
        world.insert_resource(Log::default());
        world.spawn((Position { x: 4, y: 4 }, Fox, Name("Rusty".into())));
        let near = world.spawn((Position { x: 5, y: 4 }, Rabbit));
        let far = world.spawn((Position { x: 9, y: 9 }, Rabbit));
        catch(&mut world);
        assert!(!world.is_alive(near));
        assert!(world.is_alive(far));
        assert_eq!(world.resource::<Score>().caught, 1);
        assert_eq!(
            world.resource::<Log>().0,
            ["tick 9: Rusty caught a rabbit at (5, 4)"]
        );
    }

    #[test]
    fn the_same_seed_gives_the_same_story() {
        let run = |seed| {
            let mut world = new_world(seed);
            let schedule = schedule();
            for _ in 0..40 {
                schedule.run(&mut world);
            }
            (render(&world), world.resource::<Score>().caught)
        };
        assert_eq!(run(42), run(42));
        let (_, caught) = run(42);
        assert!(caught > 0, "in 40 ticks the fox catches something");
    }

    #[test]
    fn render_draws_the_field() {
        let world = new_world(5);
        let frame = render(&world);
        let lines: Vec<&str> = frame.lines().collect();
        assert_eq!(lines[0], "tick   0  rabbits 3  caught 0");
        assert_eq!(lines.len(), 1 + 8);
        assert!(
            lines[1].starts_with('F'),
            "the fox starts in the corner: {}",
            lines[1]
        );
        // One 'r' per square with a rabbit on it (two rabbits may share one,
        // and the fox is drawn over a rabbit on its square).
        let mut squares: Vec<(i32, i32)> = world
            .with::<Rabbit>()
            .into_iter()
            .filter_map(|r| world.get::<Position>(r).map(|p| (p.x, p.y)))
            .filter(|&square| square != (0, 0))
            .collect();
        squares.sort();
        squares.dedup();
        let drawn: usize = lines[1..].iter().map(|l| l.matches('r').count()).sum();
        assert_eq!(drawn, squares.len());
    }
}
