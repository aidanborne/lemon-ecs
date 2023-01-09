use std::any::TypeId;

use lemon_ecs::world::World;

mod common;
use common::components::{Position, Velocity};

#[test]
pub fn get_component() {
    let mut world = World::new();

    let entity = world.spawn(Position(1, 2));

    let position = world.get_component::<Position>(entity);
    let velocity = world.get_component::<Velocity>(entity);

    assert_eq!(position, Some(&Position(1, 2)), "Position should be (1, 2)");
    assert_eq!(velocity, None, "Velocity should be None");

    world.remove(entity, &[TypeId::of::<Position>()]);

    let position = world.get_component::<Position>(entity);

    assert_eq!(position, None, "Position should be None");
}
