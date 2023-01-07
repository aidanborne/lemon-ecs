use lemon_ecs_macros::{Bundleable, Component};

use crate::{
    engine::Engine,
    query::{Query, Without},
    system::ResMut,
    world::{World, WorldBuffer},
};

/// Needed to make the macros work
extern crate self as lemon_ecs;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
struct Position(u32, u32);

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
struct Velocity(u32, u32);

#[derive(Bundleable)]
struct Movable(Position, Velocity);

#[test]
pub fn world_get_component() {
    let mut world = World::new();

    let entity = world.spawn(Position(1, 2));

    let position = world.get_component::<Position>(entity);
    let velocity = world.get_component::<Velocity>(entity);

    assert_eq!(position, Some(&Position(1, 2)), "Position should be (1, 2)");
    assert!(velocity.is_none(), "Velocity should be None");

    world.remove(entity, &[std::any::TypeId::of::<Position>()]);

    let position = world.get_component::<Position>(entity);
    assert!(position.is_none(), "Position should be None");
}

#[test]
pub fn world_query_no_filters() {
    let mut world = World::new();

    let _entity = world.spawn(Position(1, 2));

    let mut query = world.query::<&Position, ()>();

    let position = query.next().unwrap();
    assert_eq!(position, &Position(1, 2), "Position should be (1, 2)");
    assert!(query.next().is_none(), "Query should be empty");

    let mut query = world.query::<(&Position, &Velocity), ()>();
    assert!(query.next().is_none(), "Query should be empty");
}

#[test]
pub fn world_query_filters() {
    let mut world = World::new();

    let _entity = world.spawn((Position(1, 2), Velocity(3, 4)));

    let mut query = world.query::<&Position, ()>();

    let position = query.next().unwrap();
    assert_eq!(position, &Position(1, 2), "Position should be (1, 2)");
    assert!(query.next().is_none(), "Query should be empty");

    let mut query = world.query::<&Position, Without<Velocity>>();
    assert!(query.next().is_none(), "Query should be empty");
}

#[test]
pub fn world_query_changed() {
    let mut world = World::new();

    let _entity_a = world.spawn(Position(1, 2));
    let entity_b = world.spawn(Position(3, 4));

    world.query_changed::<Position>();

    world.insert(entity_b, Position(5, 6));
    world.insert(entity_b, Position(7, 8));

    let mut query = world.query_changed::<Position>();

    let record = query.next().unwrap();

    assert_eq!(record.id(), entity_b, "Entity should be entity_b");

    assert_eq!(
        record.removed().unwrap(),
        &Position(3, 4),
        "Removed Position should be (3, 4)"
    );
    assert_eq!(
        record.current().unwrap(),
        &Position(7, 8),
        "Added Position should be (5, 6)"
    );

    assert!(query.next().is_none(), "Query should be empty");
}

#[test]
pub fn world_multiple_entities() {
    let mut world = World::new();

    let _entity1 = world.spawn(Position(1, 2));

    let _entity2 = world.spawn(Position(3, 4));

    let mut query = world.query::<&Position, ()>();

    let position = query.next().unwrap();
    assert_eq!(position, &Position(1, 2), "Position should be (1, 2)");

    let position = query.next().unwrap();
    assert_eq!(position, &Position(3, 4), "Position should be (3, 4)");

    assert!(query.next().is_none(), "Query should be empty");
}

#[derive(Component)]
struct Name(String);

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

fn counter_system(mut counter: ResMut<Counter>) {
    counter.increment();
}

#[test]
pub fn engine_resource() {
    let mut engine = Engine::new();
    engine.add_system(counter_system);

    engine.insert_resource(Counter::new());

    for _ in 0..15 {
        engine.update();
    }

    let counter = engine.get_resource::<Counter>();

    assert_eq!(counter.unwrap().count, 15, "Counter should be 15");
}
