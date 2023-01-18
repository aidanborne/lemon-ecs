use lemon_ecs::{query::Without, world::World};

mod common;
use common::components::{Position, Velocity};

#[test]
pub fn query_no_filters() {
    let mut world = World::default();

    world.spawn(Position(1, 2));

    let mut query = world.query::<&Position, ()>();

    let position = query.next().unwrap();
    assert_eq!(position, &Position(1, 2), "Position should be (1, 2)");
    assert!(query.next().is_none(), "Query should be empty");

    let mut query = world.query::<(&Position, &Velocity), ()>();
    assert!(query.next().is_none(), "Query should be empty");
}

#[test]
pub fn query_filters() {
    let mut world = World::default();

    let _entity = world.spawn((Position(1, 2), Velocity(3, 4)));

    let mut query = world.query::<&Position, ()>();

    let position = query.next().unwrap();
    assert_eq!(position, &Position(1, 2), "Position should be (1, 2)");
    assert!(query.next().is_none(), "Query should be empty");

    let mut query = world.query::<&Position, Without<Velocity>>();
    assert!(query.next().is_none(), "Query should be empty");
}

/*#[test]
pub fn query_changed() {
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
*/
