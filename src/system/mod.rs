use crate::world::World;

pub trait System {
    fn update(&mut self, world: &mut World);
}

impl<F: FnMut(&mut World)> System for F {
    fn update(&mut self, world: &mut World) {
        self(world);
    }
}
