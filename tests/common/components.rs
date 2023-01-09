use lemon_ecs::macros::*;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Position(pub u32, pub u32);

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Velocity(pub u32, pub u32);

#[derive(Bundleable)]
pub struct Movable(pub Position, pub Velocity);

#[derive(Component)]
pub struct Name(pub String);
