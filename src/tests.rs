use lemon_ecs_macros::Component;

use crate::{world::World};

use crate::component::Component;

#[derive(Component)]
struct Position(u32, u32);

#[derive(Component)]
struct Velocity(u32, u32);

#[test]
pub fn world_get_component() {
    let mut world = World::new();
    let entity = world.spawn();

    world.add_component::<Position>(entity, Position(1, 2));
    world.add_component::<Velocity>(entity, Velocity(3, 4));

    let position = world.get_component::<Position>(entity).unwrap();
    let velocity = world.get_component::<Velocity>(entity).unwrap();

    assert_eq!(position.0, 1, "Position x should be 1");
    assert_eq!(position.1, 2, "Position y should be 2");

    assert_eq!(velocity.0, 3, "Velocity x should be 3");
    assert_eq!(velocity.1, 4, "Velocity y should be 4");
}

#[test]
pub fn world_query() {
    let mut world = World::new();

    let entity = world.spawn();
    world.add_component::<Position>(entity, Position(1, 2));

    let mut query = world.query::<Position>();
    assert!(
        query.next().is_some(),
        "Query should return an entity because the Position component is present"
    );

    let mut query = world.query::<(Position, Velocity)>();
    assert!(
        query.next().is_none(),
        "Query should not return any entities because the Velocity component is missing"
    );
}
