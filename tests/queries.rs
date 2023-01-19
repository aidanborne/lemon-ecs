use lemon_ecs::{query::Without, world::World};

mod common;
use common::components::{Position, Velocity};

#[test]
pub fn query_no_filters() {
    let mut world = World::default();

    world.spawn(Position(1, 2));

    let mut query = world.query::<Position>().into_iter();

    let position = query.next().unwrap();
    assert_eq!(position, &Position(1, 2), "Position should be (1, 2)");
    assert!(query.next().is_none(), "Query should be empty");

    let mut query = world.query::<(Position, Velocity)>().into_iter();
    assert!(query.next().is_none(), "Query should be empty");
}

#[test]
pub fn query_filters() {
    let mut world = World::default();

    let _entity = world.spawn((Position(1, 2), Velocity(3, 4)));

    let mut query = world.query::<Position>().into_iter();

    let position = query.next().unwrap();
    assert_eq!(position, &Position(1, 2), "Position should be (1, 2)");
    assert!(query.next().is_none(), "Query should be empty");

    let mut query = world.query::<Position>().filter::<Without<Velocity>>();
    assert!(query.next().is_none(), "Query should be empty");
}

#[test]
pub fn query_changed_snapshot() {
    let mut world = World::default();

    let _entity_a = world.spawn(Position(1, 2));
    let entity_b = world.spawn(Position(3, 4));

    world.query_changed::<Position>();

    world.insert(entity_b, Position(5, 6));
    world.insert(entity_b, Position(7, 8));

    let mut query = world.query_changed::<Position>().into_iter();

    let record = query.next().unwrap();

    assert_eq!(record.id(), entity_b, "Entity should be entity_b");

    assert_eq!(
        record.get_old().unwrap(),
        &Position(3, 4),
        "Removed Position should be (3, 4)"
    );

    assert_eq!(
        record.get_new().unwrap(),
        &Position(7, 8),
        "Added Position should be (5, 6)"
    );

    assert!(query.next().is_none(), "Query should be empty");
}

#[test]
pub fn query_changed_added() {
    let mut world = World::default();

    let entity_a = world.spawn(Position(1, 2));
    let entity_b = world.spawn(Position(3, 4));

    world.query_changed::<Position>();

    world.insert(entity_a, Position(5, 6));
    world.insert(entity_b, Position(7, 8));

    let mut query = world.query_changed::<Position>().added();

    let record = query.next().unwrap();

    assert_eq!(record.0, entity_a, "Entity should be entity_a");
    assert_eq!(record.1, &Position(5, 6), "Added Position should be (5, 6)");

    let record = query.next().unwrap();

    assert_eq!(record.0, entity_b, "Entity should be entity_b");
    assert_eq!(record.1, &Position(7, 8), "Added Position should be (7, 8)");

    assert!(query.next().is_none(), "Query should be empty");
}

#[test]
pub fn query_changed_removed() {
    let mut world = World::default();

    let entity_a = world.spawn(Position(1, 2));
    let entity_b = world.spawn(Position(3, 4));

    world.query_changed::<Position>();

    world.remove::<Position>(entity_a);
    world.remove::<Position>(entity_b);

    let mut query = world.query_changed::<Position>().removed();

    let record = query.next().unwrap();

    assert_eq!(record.0, entity_a, "Entity should be entity_a");

    assert_eq!(
        record.1,
        Position(1, 2),
        "Removed Position should be (1, 2)"
    );

    let record = query.next().unwrap();

    assert_eq!(record.0, entity_b, "Entity should be entity_b");

    assert_eq!(
        record.1,
        Position(3, 4),
        "Removed Position should be (3, 4)"
    );

    assert!(query.next().is_none(), "Query should be empty");
}

#[test]
pub fn query_changed_modified() {
    let mut world = World::default();

    let entity_a = world.spawn(Position(1, 2));
    let entity_b = world.spawn(Position(3, 4));

    world.query_changed::<Position>();

    world.insert(entity_a, Position(5, 6));
    world.insert(entity_b, Position(7, 8));

    let mut query = world.query_changed::<Position>().modified();

    let record = query.next().unwrap();

    assert_eq!(record.0, entity_a, "Entity should be entity_a");

    assert_eq!(
        record.1,
        Position(1, 2),
        "Removed Position should be (1, 2)"
    );

    assert_eq!(record.2, &Position(5, 6), "Added Position should be (5, 6)");

    let record = query.next().unwrap();

    assert_eq!(record.0, entity_b, "Entity should be entity_b");

    assert_eq!(
        record.1,
        Position(3, 4),
        "Removed Position should be (3, 4)"
    );

    assert_eq!(record.2, &Position(7, 8), "Added Position should be (7, 8)");

    assert!(query.next().is_none(), "Query should be empty");
}
