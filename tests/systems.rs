use lemon_ecs::{
    engine::Engine,
    world::{World, WorldBuffer},
};

mod common;
use common::components::{Name, Position, Velocity};

fn print_system(world: &mut World) {
    for (mut position, velocity) in world.query::<(&mut Position, &Velocity), ()>() {
        let position = &mut *position;
        position.0 += velocity.0;
        position.1 += velocity.1;

        WorldBuffer::new(world).spawn((Name("Hello".to_string()), Velocity(3, 4)));
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

#[test]
pub fn engine_resource() {
    let mut engine = Engine::new();
    engine.insert_resource(Counter::new());

    for _ in 0..15 {
        if let Some(counter) = engine.get_resource_mut::<Counter>() {
            counter.increment();
        }
    }

    let counter = engine.get_resource::<Counter>();

    assert_eq!(counter.unwrap().count, 15, "Counter should be 15");
}
