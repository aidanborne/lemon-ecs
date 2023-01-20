use crate::world::World;

pub trait System {
    fn run(&mut self, world: &mut World);
}

impl<F: FnMut(&mut World)> System for F {
    fn run(&mut self, world: &mut World) {
        self(world);
    }
}

impl System for Vec<Box<dyn System>> {
    #[inline]
    fn run(&mut self, world: &mut World) {
        for system in self.iter_mut() {
            system.run(world);
        }
    }
}
