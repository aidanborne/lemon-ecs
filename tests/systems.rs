use lemon_ecs::{engine::Engine, query::Query, system::Resource, world::WorldBuffer};

mod common;
use common::components::{Name, Position, Velocity};
use lemon_ecs_macros::impl_as_any;

fn print_system(buffer: WorldBuffer, query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in query {
        let position = &mut *position;
        position.0 += velocity.0;
        position.1 += velocity.1;

        buffer
            .spawn(Name("Hello".to_string()))
            .insert(Velocity(3, 4));
    }
}

#[test]
pub fn engine_run() {
    let mut engine = Engine::new();
    engine.add_system(print_system);

    let entity = engine.spawn((Position(1, 2), Velocity(3, 4)));

    for _ in 0..10 {
        engine.update();
    }

    let position = engine.get_component::<Position>(entity);

    assert_eq!(
        position.unwrap(),
        &Position(31, 42),
        "Position should be (31, 42)"
    );

    let mut query = engine.query::<(&Name, &Velocity), ()>();

    let (name, velocity) = query.next().unwrap();

    assert_eq!(name.0, "Hello", "Name should be 'Hello'");
    assert_eq!(velocity, &Velocity(3, 4), "Velocity should be (3, 4)");
}

#[derive(Clone)]
struct Counter {
    count: u32,
}

impl Counter {
    fn new() -> Self {
        Self { count: 0 }
    }

    fn increment(&mut self) {
        self.count += 1;
    }
}

impl Resource for Counter {
    fn update(&mut self) {
        self.increment();
    }
}

impl_as_any!(Counter);

#[test]
pub fn engine_resource() {
    let mut engine = Engine::new();
    engine.insert_resource(Counter::new());

    for _ in 0..15 {
        engine.update();
    }

    let counter = engine.get_resource::<Counter>();

    assert_eq!(counter.unwrap().count, 15, "Counter should be 15");
}
