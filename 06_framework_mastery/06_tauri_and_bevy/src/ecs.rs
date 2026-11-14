//! A tiny Entity-Component-System, built the way Bevy's works inside (much
//! simplified): entities are ids, components live in one storage per type,
//! systems are plain functions run by a schedule, and resources are
//! one-of-a-kind values shared by all systems.

use std::any::{Any, TypeId, type_name};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

/// An entity is just an id: it has no data of its own. Its components are
/// stored elsewhere, by type.
///
/// The `generation` makes old ids safe: when an entity is despawned, its
/// index can be reused, but with a new generation, so a stale `Entity`
/// can't accidentally point at the newcomer. Bevy's `Entity` works the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity {
    index: u32,
    generation: u32,
}

impl Entity {
    /// The slot this entity uses in every storage.
    pub fn index(self) -> usize {
        self.index as usize
    }

    pub fn generation(self) -> u32 {
        self.generation
    }
}

/// One storage per component type: slot `i` holds entity `i`'s component.
pub struct Storage<T> {
    slots: Vec<Option<T>>,
}

impl<T> Storage<T> {
    fn set(&mut self, index: usize, value: T) {
        if self.slots.len() <= index {
            self.slots.resize_with(index + 1, || None);
        }
        self.slots[index] = Some(value);
    }

    /// The component in slot `index`, if there is one.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.slots.get(index)?.as_ref()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.slots.get_mut(index)?.as_mut()
    }
}

/// Lets the world treat every `Storage<T>` alike, whatever `T` is ("type
/// erasure"): it can remove a despawned entity's component without knowing
/// the type, and hand the real `Storage<T>` back with a downcast.
trait AnyStorage {
    fn remove(&mut self, index: usize);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> AnyStorage for Storage<T> {
    fn remove(&mut self, index: usize) {
        if let Some(slot) = self.slots.get_mut(index) {
            *slot = None;
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Everything: entities, their components, and the resources.
///
/// Storages and resources sit in `RefCell`s, so systems can read one
/// storage and write another at the same time through `&World`. Asking for
/// the SAME storage mutably twice panics: that's the conflict Bevy refuses
/// with error B0001 when you build the app.
#[derive(Default)]
pub struct World {
    generations: Vec<u32>,
    alive: Vec<bool>,
    free: Vec<u32>,
    storages: HashMap<TypeId, RefCell<Box<dyn AnyStorage>>>,
    resources: HashMap<TypeId, RefCell<Box<dyn Any>>>,
}

impl World {
    pub fn new() -> Self {
        World::default()
    }

    // ---------- entities ----------

    /// A new entity with no components, reusing a freed index if there is one.
    pub fn spawn_empty(&mut self) -> Entity {
        if let Some(index) = self.free.pop() {
            let i = index as usize;
            self.generations[i] += 1;
            self.alive[i] = true;
            return Entity {
                index,
                generation: self.generations[i],
            };
        }
        let index = u32::try_from(self.generations.len()).expect("fewer than 4 billion entities");
        self.generations.push(0);
        self.alive.push(true);
        Entity {
            index,
            generation: 0,
        }
    }

    /// A new entity with a bundle of components: `world.spawn((Position { .. }, Rabbit))`,
    /// like Bevy's `commands.spawn((A, B))`.
    pub fn spawn(&mut self, bundle: impl Bundle) -> Entity {
        let entity = self.spawn_empty();
        bundle.insert_into(self, entity);
        entity
    }

    /// Whether `entity` still exists (and isn't an old id for a reused slot).
    pub fn is_alive(&self, entity: Entity) -> bool {
        let i = entity.index();
        self.alive.get(i) == Some(&true) && self.generations[i] == entity.generation
    }

    /// Removes an entity and all its components. False if it was already gone.
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }
        for storage in self.storages.values() {
            storage.borrow_mut().remove(entity.index());
        }
        self.alive[entity.index()] = false;
        self.free.push(entity.index);
        true
    }

    /// Every living entity, in index order.
    pub fn entities(&self) -> Vec<Entity> {
        (0..self.generations.len())
            .filter(|&i| self.alive[i])
            .map(|i| Entity {
                index: i as u32,
                generation: self.generations[i],
            })
            .collect()
    }

    // ---------- components ----------

    /// Adds (or replaces) a component. Any `'static` type can be a
    /// component here; Bevy asks you to `#[derive(Component)]` first.
    ///
    /// # Panics
    ///
    /// If `entity` has been despawned.
    pub fn insert<T: 'static>(&mut self, entity: Entity, component: T) {
        assert!(
            self.is_alive(entity),
            "insert into a despawned entity {entity:?}"
        );
        let storage = self
            .storages
            .entry(TypeId::of::<T>())
            .or_insert_with(|| RefCell::new(Box::new(Storage::<T> { slots: Vec::new() })));
        storage
            .get_mut()
            .as_any_mut()
            .downcast_mut::<Storage<T>>()
            .expect("the storage for T holds T")
            .set(entity.index(), component);
    }

    /// Removes one component from an entity.
    pub fn remove<T: 'static>(&mut self, entity: Entity) {
        if self.is_alive(entity)
            && let Some(storage) = self.storages.get_mut(&TypeId::of::<T>())
        {
            storage.get_mut().remove(entity.index());
        }
    }

    /// Read access to every `T` (like a `Query<&T>`). None if no entity ever had one.
    pub fn read<T: 'static>(&self) -> Option<Ref<'_, Storage<T>>> {
        let cell = self.storages.get(&TypeId::of::<T>())?;
        Some(Ref::map(cell.borrow(), |s| {
            s.as_any()
                .downcast_ref::<Storage<T>>()
                .expect("storage type")
        }))
    }

    /// Write access to every `T` (like a `Query<&mut T>`).
    ///
    /// # Panics
    ///
    /// If the same storage is already borrowed (two mutable queries of one type).
    pub fn write<T: 'static>(&self) -> Option<RefMut<'_, Storage<T>>> {
        let cell = self.storages.get(&TypeId::of::<T>())?;
        Some(RefMut::map(cell.borrow_mut(), |s| {
            s.as_any_mut()
                .downcast_mut::<Storage<T>>()
                .expect("storage type")
        }))
    }

    /// A copy of one entity's component.
    pub fn get<T: Clone + 'static>(&self, entity: Entity) -> Option<T> {
        if !self.is_alive(entity) {
            return None;
        }
        self.read::<T>()?.get(entity.index()).cloned()
    }

    /// Whether the entity has a `T`.
    pub fn has<T: 'static>(&self, entity: Entity) -> bool {
        self.is_alive(entity)
            && self
                .read::<T>()
                .is_some_and(|storage| storage.get(entity.index()).is_some())
    }

    // ---------- queries ----------

    /// The living entities that have a `T`: `Query<Entity, With<T>>`.
    pub fn with<T: 'static>(&self) -> Vec<Entity> {
        self.entities()
            .into_iter()
            .filter(|&e| self.has::<T>(e))
            .collect()
    }

    /// Runs `f` for every entity with an `A`: `for a in &query`.
    pub fn each<A: 'static>(&self, mut f: impl FnMut(Entity, &A)) {
        let Some(a) = self.read::<A>() else { return };
        for entity in self.entities() {
            if let Some(value) = a.get(entity.index()) {
                f(entity, value);
            }
        }
    }

    /// Runs `f` for every entity with BOTH an `A` (changeable) and a `B`
    /// (read-only): Bevy's `Query<(&mut A, &B)>`.
    ///
    /// # Panics
    ///
    /// If `A` and `B` are the same type: that would be a `&mut` and a `&`
    /// to the same data at once (Bevy's error B0001).
    pub fn each2_mut<A: 'static, B: 'static>(&self, mut f: impl FnMut(Entity, &mut A, &B)) {
        assert_ne!(
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            "conflicting access: &mut {0} and &{0} in one query",
            type_name::<A>()
        );
        let (Some(mut a), Some(b)) = (self.write::<A>(), self.read::<B>()) else {
            return;
        };
        for entity in self.entities() {
            if let (Some(x), Some(y)) = (a.get_mut(entity.index()), b.get(entity.index())) {
                f(entity, x, y);
            }
        }
    }

    // ---------- resources ----------

    /// Stores a one-of-a-kind value (a score, the time, settings...).
    pub fn insert_resource<R: 'static>(&mut self, resource: R) {
        self.resources
            .insert(TypeId::of::<R>(), RefCell::new(Box::new(resource)));
    }

    /// Reads a resource: Bevy's `Res<R>`.
    ///
    /// # Panics
    ///
    /// If there is no `R`, with its type name (Bevy panics the same way).
    pub fn resource<R: 'static>(&self) -> Ref<'_, R> {
        let cell = self
            .resources
            .get(&TypeId::of::<R>())
            .unwrap_or_else(|| panic!("resource {} does not exist", type_name::<R>()));
        Ref::map(cell.borrow(), |r| {
            r.downcast_ref::<R>().expect("resource type")
        })
    }

    /// Changes a resource: Bevy's `ResMut<R>`.
    ///
    /// # Panics
    ///
    /// If there is no `R`, or it is already borrowed.
    pub fn resource_mut<R: 'static>(&self) -> RefMut<'_, R> {
        let cell = self
            .resources
            .get(&TypeId::of::<R>())
            .unwrap_or_else(|| panic!("resource {} does not exist", type_name::<R>()));
        RefMut::map(cell.borrow_mut(), |r| {
            r.downcast_mut::<R>().expect("resource type")
        })
    }
}

/// Several components inserted together: implemented for tuples of up to
/// four components by the macro below (Bevy goes up to 16 the same way).
pub trait Bundle {
    fn insert_into(self, world: &mut World, entity: Entity);
}

macro_rules! impl_bundle {
    ($($name:ident),+) => {
        impl<$($name: 'static),+> Bundle for ($($name,)+) {
            #[allow(non_snake_case)]
            fn insert_into(self, world: &mut World, entity: Entity) {
                let ($($name,)+) = self;
                $(world.insert(entity, $name);)+
            }
        }
    };
}

impl_bundle!(A);
impl_bundle!(A, B);
impl_bundle!(A, B, C);
impl_bundle!(A, B, C, D);

/// Changes to make AFTER the current loop: spawning and despawning while a
/// storage is borrowed would be a conflict, so systems queue them here.
/// Bevy's `Commands` work the same way and are applied between systems.
#[derive(Default)]
pub struct Commands {
    queue: Vec<Change>,
}

/// One queued change to the world.
type Change = Box<dyn FnOnce(&mut World)>;

impl Commands {
    pub fn spawn(&mut self, bundle: impl Bundle + 'static) {
        self.queue.push(Box::new(move |world| {
            world.spawn(bundle);
        }));
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.queue.push(Box::new(move |world| {
            world.despawn(entity);
        }));
    }

    /// Runs the queued changes, in order.
    pub fn apply(self, world: &mut World) {
        for change in self.queue {
            change(world);
        }
    }
}

/// A system: a plain function that works on the world.
pub type System = fn(&mut World);

/// The systems to run each frame, in order. Bevy also works out which
/// systems touch different data and runs those in parallel; this runs
/// them one after another.
#[derive(Default)]
pub struct Schedule {
    systems: Vec<(&'static str, System)>,
}

impl Schedule {
    pub fn new() -> Self {
        Schedule::default()
    }

    /// Adds a system at the end: `app.add_systems(Update, name)`.
    pub fn add(mut self, name: &'static str, system: System) -> Self {
        self.systems.push((name, system));
        self
    }

    /// The systems' names, in order.
    pub fn names(&self) -> Vec<&'static str> {
        self.systems.iter().map(|(name, _)| *name).collect()
    }

    /// Runs every system once: one frame (or "tick").
    pub fn run(&self, world: &mut World) {
        for (_, system) in &self.systems {
            system(world);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Position(i32);
    #[derive(Debug, Clone, PartialEq)]
    struct Velocity(i32);
    struct Frozen;

    #[test]
    fn spawn_insert_get() {
        let mut world = World::new();
        let a = world.spawn((Position(1), Velocity(2)));
        let b = world.spawn((Position(10),));
        assert_eq!(world.get::<Position>(a), Some(Position(1)));
        assert_eq!(world.get::<Velocity>(b), None);
        assert!(world.has::<Velocity>(a));
        assert_eq!(world.with::<Velocity>(), vec![a]);
    }

    #[test]
    fn despawn_removes_components_and_reuses_the_index_safely() {
        let mut world = World::new();
        let old = world.spawn((Position(1),));
        assert!(world.despawn(old));
        assert!(!world.despawn(old), "already gone");

        let new = world.spawn((Position(99),));
        assert_eq!(new.index(), old.index(), "the slot is reused");
        assert_ne!(new, old, "but the generation differs");
        assert_eq!(
            world.get::<Position>(old),
            None,
            "the stale id sees nothing"
        );
        assert_eq!(world.get::<Position>(new), Some(Position(99)));
    }

    #[test]
    fn each2_mut_only_visits_entities_with_both() {
        let mut world = World::new();
        let moving = world.spawn((Position(0), Velocity(3)));
        let still = world.spawn((Position(5),));
        world.each2_mut::<Position, Velocity>(|_, p, v| p.0 += v.0);
        assert_eq!(world.get::<Position>(moving), Some(Position(3)));
        assert_eq!(world.get::<Position>(still), Some(Position(5)));
    }

    #[test]
    #[should_panic(expected = "conflicting access")]
    fn the_same_type_twice_in_one_query_is_refused() {
        let mut world = World::new();
        world.spawn((Position(0),));
        world.each2_mut::<Position, Position>(|_, _, _| {});
    }

    #[test]
    fn remove_one_component() {
        let mut world = World::new();
        let e = world.spawn((Position(0), Frozen));
        world.remove::<Frozen>(e);
        assert!(!world.has::<Frozen>(e));
        assert!(world.has::<Position>(e));
    }

    #[test]
    fn resources_are_shared_values() {
        let mut world = World::new();
        world.insert_resource(0u32);
        *world.resource_mut::<u32>() += 5;
        assert_eq!(*world.resource::<u32>(), 5);
    }

    #[test]
    #[should_panic(expected = "resource f64 does not exist")]
    fn a_missing_resource_names_its_type() {
        let world = World::new();
        let _ = world.resource::<f64>();
    }

    #[test]
    fn commands_apply_after_the_loop() {
        let mut world = World::new();
        let a = world.spawn((Position(1),));
        let b = world.spawn((Position(-1),));
        let mut commands = Commands::default();
        world.each::<Position>(|e, p| {
            if p.0 < 0 {
                commands.despawn(e); // despawning here directly would clash with the borrow
            }
        });
        commands.spawn((Position(7),));
        commands.apply(&mut world);
        assert!(world.is_alive(a));
        assert!(!world.is_alive(b));
        assert_eq!(world.with::<Position>().len(), 2);
    }

    #[test]
    fn a_schedule_runs_systems_in_order() {
        fn double(world: &mut World) {
            *world.resource_mut::<i32>() *= 2;
        }
        fn add_one(world: &mut World) {
            *world.resource_mut::<i32>() += 1;
        }
        let mut world = World::new();
        world.insert_resource(1i32);
        let schedule = Schedule::new()
            .add("double", double)
            .add("add_one", add_one);
        schedule.run(&mut world);
        schedule.run(&mut world);
        assert_eq!(*world.resource::<i32>(), 7); // (1*2+1)*2+1
        assert_eq!(schedule.names(), ["double", "add_one"]);
    }
}
