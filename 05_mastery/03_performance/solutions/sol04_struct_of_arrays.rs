//! Solution to exercise 4: implement a struct-of-arrays world, and pass the tests.
//!
//! A game keeps thousands of `Entity` values. Every frame it moves them all
//! (reads x, y, vx, vy) and counts the living ones (reads health). With a
//! `Vec<Entity>`, each of those loops drags whole entities, name included,
//! through the CPU cache. `World` stores one `Vec` per field instead, so
//! each loop reads only the bytes it needs.
//!
//! Implement every `todo!()`. Rules:
//! - all the Vecs in a `World` always have the same length (one slot per
//!   entity, in the order they were added);
//! - `step` moves every entity: x += vx * dt and y += vy * dt;
//! - `alive` counts the entities whose health is above 0;
//! - `entity(i)` puts entity `i` back together (or None if out of range).
//!
//! Test: cargo test -p performance --features solutions --bin sol04_struct_of_arrays

#[derive(Debug, Clone, PartialEq)]
struct Entity {
    name: String,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    health: u32,
}

#[derive(Debug, Default)]
struct World {
    names: Vec<String>,
    x: Vec<f32>,
    y: Vec<f32>,
    vx: Vec<f32>,
    vy: Vec<f32>,
    health: Vec<u32>,
}

impl World {
    /// An empty world with room for `n` entities in every Vec.
    fn with_capacity(n: usize) -> World {
        World {
            names: Vec::with_capacity(n),
            x: Vec::with_capacity(n),
            y: Vec::with_capacity(n),
            vx: Vec::with_capacity(n),
            vy: Vec::with_capacity(n),
            health: Vec::with_capacity(n),
        }
    }

    /// Adds one entity at the end.
    fn push(&mut self, e: Entity) {
        self.names.push(e.name);
        self.x.push(e.x);
        self.y.push(e.y);
        self.vx.push(e.vx);
        self.vy.push(e.vy);
        self.health.push(e.health);
    }

    /// How many entities there are.
    fn len(&self) -> usize {
        self.x.len()
    }

    /// Moves every entity by its velocity times `dt`.
    fn step(&mut self, dt: f32) {
        // Two tight loops over plain f32 slices: cache-friendly, no bounds
        // checks (zip), and easy for the compiler to vectorise.
        for (x, vx) in self.x.iter_mut().zip(&self.vx) {
            *x += vx * dt;
        }
        for (y, vy) in self.y.iter_mut().zip(&self.vy) {
            *y += vy * dt;
        }
    }

    /// How many entities have health above 0.
    fn alive(&self) -> usize {
        self.health.iter().filter(|&&h| h > 0).count()
    }

    /// Entity `i`, put back together.
    fn entity(&self, i: usize) -> Option<Entity> {
        if i >= self.len() {
            return None;
        }
        Some(Entity {
            name: self.names[i].clone(),
            x: self.x[i],
            y: self.y[i],
            vx: self.vx[i],
            vy: self.vy[i],
            health: self.health[i],
        })
    }
}

fn entity(name: &str, x: f32, vx: f32, health: u32) -> Entity {
    Entity {
        name: name.to_string(),
        x,
        y: 0.0,
        vx,
        vy: 1.0,
        health,
    }
}

fn main() {
    let mut world = World::with_capacity(3);
    world.push(entity("ant", 0.0, 1.0, 5));
    world.push(entity("bee", 10.0, -2.0, 0));
    world.step(0.5);
    println!("{} entities, {} alive", world.len(), world.alive());
    println!("{:?}", world.entity(1));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn three() -> World {
        let mut world = World::with_capacity(3);
        world.push(entity("ant", 0.0, 1.0, 5));
        world.push(entity("bee", 10.0, -2.0, 0));
        world.push(entity("cat", 3.0, 0.0, 9));
        world
    }

    #[test]
    fn push_keeps_every_vec_the_same_length() {
        let world = three();
        assert_eq!(world.len(), 3);
        for len in [
            world.x.len(),
            world.y.len(),
            world.vx.len(),
            world.vy.len(),
            world.health.len(),
        ] {
            assert_eq!(len, 3);
        }
        assert!(World::with_capacity(100).x.capacity() >= 100);
    }

    #[test]
    fn step_moves_by_velocity_times_dt() {
        let mut world = three();
        world.step(0.5);
        assert_eq!(world.x, [0.5, 9.0, 3.0]);
        assert_eq!(world.y, [0.5, 0.5, 0.5]);
    }

    #[test]
    fn alive_counts_positive_health() {
        assert_eq!(three().alive(), 2);
        assert_eq!(World::default().alive(), 0);
    }

    #[test]
    fn entity_puts_one_back_together() {
        let mut world = three();
        world.step(1.0);
        let mut bee = entity("bee", 8.0, -2.0, 0);
        bee.y = 1.0;
        assert_eq!(world.entity(1), Some(bee));
        assert_eq!(world.entity(3), None);
    }
}
