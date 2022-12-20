use lemon_ecs_macros::Component;

use crate::{
    query::{filter::Without, Query},
    world::World,
};

/// Needed to make the macros work
extern crate self as lemon_ecs;

#[derive(Component, PartialEq, Eq, Debug)]
struct Position(u32, u32);

#[derive(Component, PartialEq, Eq, Debug)]
struct Velocity(u32, u32);

#[test]
pub fn world_get_component() {
    let mut world = World::new();
    let entity = world.spawn();

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

    let entity = world.spawn();
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

    let entity = world.spawn();
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

    let entity1 = world.spawn();
    world.add_component::<Position>(entity1, Position(1, 2));

    let entity2 = world.spawn();
    world.add_component::<Position>(entity2, Position(3, 4));

    let mut query = world.query::<Position>();

    let position = query.next().unwrap();
    assert_eq!(position, &Position(1, 2), "Position should be (1, 2)");

    let position = query.next().unwrap();
    assert_eq!(position, &Position(3, 4), "Position should be (3, 4)");

    assert!(query.next().is_none(), "Query should be empty");
}
