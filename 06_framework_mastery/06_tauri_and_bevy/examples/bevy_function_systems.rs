//! How can Bevy call `fn score_system(time: Res<Time>, mut score: ResMut<Score>)`
//! when every system has different parameters? This example rebuilds the
//! trick in about 100 lines: a trait for "things a system can ask for"
//! (`SystemParam`), and a trait implemented for every FUNCTION whose
//! parameters are all such things (`IntoSystem`).
//! Run: cargo run -p tauri_and_bevy --example bevy_function_systems

use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Resources by type, as in our World.
#[derive(Default)]
struct Resources {
    map: HashMap<TypeId, RefCell<Box<dyn Any>>>,
}

impl Resources {
    fn insert<T: 'static>(&mut self, value: T) {
        self.map
            .insert(TypeId::of::<T>(), RefCell::new(Box::new(value)));
    }
    fn cell<T: 'static>(&self) -> &RefCell<Box<dyn Any>> {
        self.map
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| panic!("no resource {}", std::any::type_name::<T>()))
    }
}

/// Read access to a resource, like Bevy's `Res<T>`. (A named field, not a
/// tuple field: with `Res(Ref<T>)`, `score.0` would mean Res's own field 0,
/// not the resource's.)
struct Res<'w, T: 'static> {
    value: Ref<'w, T>,
}
/// Write access, like Bevy's `ResMut<T>`.
struct ResMut<'w, T: 'static> {
    value: RefMut<'w, T>,
}

impl<T> Deref for Res<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
impl<T> Deref for ResMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
impl<T> DerefMut for ResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

/// Something a system can ask for in its parameter list. `Item<'w>` is the
/// same type borrowed from a particular world, for the length of one run.
trait SystemParam {
    type Item<'w>;
    fn fetch(resources: &Resources) -> Self::Item<'_>;
}

impl<T: 'static> SystemParam for Res<'_, T> {
    type Item<'w> = Res<'w, T>;
    fn fetch(resources: &Resources) -> Res<'_, T> {
        Res {
            value: Ref::map(resources.cell::<T>().borrow(), |b| {
                b.downcast_ref().expect("type")
            }),
        }
    }
}

impl<T: 'static> SystemParam for ResMut<'_, T> {
    type Item<'w> = ResMut<'w, T>;
    fn fetch(resources: &Resources) -> ResMut<'_, T> {
        ResMut {
            value: RefMut::map(resources.cell::<T>().borrow_mut(), |b| {
                b.downcast_mut().expect("type")
            }),
        }
    }
}

/// Anything the schedule can run.
trait System {
    fn run(&mut self, resources: &Resources);
}

/// A function plus a note of its parameter types (the "marker").
struct FunctionSystem<F, Params> {
    f: F,
    _params: PhantomData<fn() -> Params>,
}

/// "Turn me into a system". Implemented below for functions with one and
/// two parameters; Bevy uses a macro to cover up to 16.
trait IntoSystem<Params> {
    fn into_system(self) -> Box<dyn System>;
}

// The key line: `for<'a> &'a mut F: FnMut(P1::Item<'_>)`. A function
// written `fn sys(t: Res<Time>)` works for ANY lifetime of the borrow, so it
// can be called with whatever `fetch` returns this time.
impl<F, P1> System for FunctionSystem<F, (P1,)>
where
    P1: SystemParam,
    for<'a> &'a mut F: FnMut(P1) + FnMut(P1::Item<'_>),
{
    fn run(&mut self, resources: &Resources) {
        // Calls through `&mut F` to pick the second FnMut bound.
        fn call<P1>(mut f: impl FnMut(P1), p1: P1) {
            f(p1)
        }
        call(&mut self.f, P1::fetch(resources));
    }
}

impl<F, P1, P2> System for FunctionSystem<F, (P1, P2)>
where
    P1: SystemParam,
    P2: SystemParam,
    for<'a> &'a mut F: FnMut(P1, P2) + FnMut(P1::Item<'_>, P2::Item<'_>),
{
    fn run(&mut self, resources: &Resources) {
        fn call<P1, P2>(mut f: impl FnMut(P1, P2), p1: P1, p2: P2) {
            f(p1, p2)
        }
        call(&mut self.f, P1::fetch(resources), P2::fetch(resources));
    }
}

impl<F: 'static, P1: 'static> IntoSystem<(P1,)> for F
where
    FunctionSystem<F, (P1,)>: System,
{
    fn into_system(self) -> Box<dyn System> {
        Box::new(FunctionSystem::<F, (P1,)> {
            f: self,
            _params: PhantomData,
        })
    }
}

impl<F: 'static, P1: 'static, P2: 'static> IntoSystem<(P1, P2)> for F
where
    FunctionSystem<F, (P1, P2)>: System,
{
    fn into_system(self) -> Box<dyn System> {
        Box::new(FunctionSystem::<F, (P1, P2)> {
            f: self,
            _params: PhantomData,
        })
    }
}

/// A tiny `App`: resources plus a list of systems.
#[derive(Default)]
struct App {
    resources: Resources,
    systems: Vec<Box<dyn System>>,
}

impl App {
    fn insert_resource<T: 'static>(mut self, value: T) -> Self {
        self.resources.insert(value);
        self
    }
    fn add_system<Params>(mut self, system: impl IntoSystem<Params>) -> Self {
        self.systems.push(system.into_system());
        self
    }
    fn update(&mut self) {
        for system in &mut self.systems {
            system.run(&self.resources);
        }
    }
}

// ---------- user code: looks just like Bevy ----------

struct Time {
    seconds: f64,
}
struct Score(u32);

fn advance_time(mut time: ResMut<Time>) {
    time.seconds += 0.5;
}

fn add_points(time: Res<Time>, mut score: ResMut<Score>) {
    score.0 += (time.seconds * 10.0) as u32;
}

fn report(score: Res<Score>) {
    println!("score is now {}", score.0);
}

fn main() {
    let mut app = App::default()
        .insert_resource(Time { seconds: 0.0 })
        .insert_resource(Score(0))
        .add_system(advance_time)
        .add_system(add_points)
        .add_system(report);
    for frame in 1..=3 {
        println!("frame {frame}:");
        app.update();
    }
    // Asking for the same resource twice, as Res AND ResMut, compiles but
    // panics when it runs (RefCell). Bevy checks this when the app starts:
    // error B0002.
}
