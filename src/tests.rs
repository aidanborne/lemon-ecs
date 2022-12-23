use lemon_ecs_macros::{Bundleable, Component};

use crate::{
    engine::Engine,
    query::{filter::Without, Query},
    system::buffer::SystemBuffer,
    world::World,
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
    let entity = world.spawn_empty();

    world.add_component::<Position>(entity, Position(1, 2));

    let position = world.get_component::<Position>(entity);
    let velocity = world.get_component::<Velocity>(entity);

    assert_eq!(
        position.unwrap(),
        &Position(1, 2),
        "Position should be (1, 2)"
    );
    assert!(velocity.is_none(), "Velocity should be None");
}

#[test]
pub fn world_query_basic() {
    let mut world = World::new();

    let entity = world.spawn_empty();
    world.add_component::<Position>(entity, Position(1, 2));

    let mut query = world.query::<Position>();

    let position = query.next().unwrap();
    assert_eq!(position, &Position(1, 2), "Position should be (1, 2)");
    assert!(query.next().is_none(), "Query should be empty");

    let mut query = world.query::<(Position, Velocity)>();
    assert!(query.next().is_none(), "Query should be empty");
}

#[test]
pub fn world_query_filters() {
    let mut world = World::new();

    let entity = world.spawn_empty();
    world.add_component::<Position>(entity, Position(1, 2));
    world.add_component::<Velocity>(entity, Velocity(3, 4));

    let mut query = world.query::<Query<Position>>();

    let position = query.next().unwrap();
    assert_eq!(position, &Position(1, 2), "Position should be (1, 2)");
    assert!(query.next().is_none(), "Query should be empty");

    let mut query = world.query::<Query<Position, Without<Velocity>>>();
    assert!(query.next().is_none(), "Query should be empty");
}

#[test]
pub fn world_multiple_entities() {
    let mut world = World::new();

    let entity1 = world.spawn_empty();
    world.add_component::<Position>(entity1, Position(1, 2));

    let entity2 = world.spawn_empty();
    world.add_component::<Position>(entity2, Position(3, 4));

    let mut query = world.query::<Position>();

    let position = query.next().unwrap();
    assert_eq!(position, &Position(1, 2), "Position should be (1, 2)");

    let position = query.next().unwrap();
    assert_eq!(position, &Position(3, 4), "Position should be (3, 4)");

    assert!(query.next().is_none(), "Query should be empty");
}

fn print_system(buffer: SystemBuffer, query: Query<(usize, Position, Velocity)>) {
    for (id, position, velocity) in query {
        buffer.insert(
            id,
            Position(position.0 + velocity.0, position.1 + velocity.1),
        );
    }
}

#[test]
pub fn engine_run() {
    let mut engine = Engine::new();
    engine.add_system(print_system);

    let entity = engine.spawn_empty();

    engine.add_component::<Position>(entity, Position(1, 2));

    engine.add_component::<Velocity>(entity, Velocity(3, 4));

    for _ in 0..10 {
        engine.update();
    }

    let position = engine.get_component::<Position>(entity);

    assert_eq!(
        position,
        Some(&Position(31, 42)),
        "Position should be (31, 42)"
    );
}
