use std::ops::{Deref, DerefMut};

use crate::component::Component;

#[repr(transparent)]
#[derive(Default)]
pub struct ComponentBundle {
    components: Vec<Box<dyn Component>>,
}

impl ComponentBundle {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }
}

impl IntoIterator for ComponentBundle {
    type Item = Box<dyn Component>;
    type IntoIter = std::vec::IntoIter<Box<dyn Component>>;

    fn into_iter(self) -> Self::IntoIter {
        self.components.into_iter()
    }
}

impl FromIterator<Box<dyn Component>> for ComponentBundle {
    fn from_iter<I: IntoIterator<Item = Box<dyn Component>>>(iter: I) -> Self {
        let mut components = Vec::new();

        for component in iter {
            components.push(component);
        }

        Self { components }
    }
}

impl Deref for ComponentBundle {
    type Target = Vec<Box<dyn Component>>;

    fn deref(&self) -> &Self::Target {
        &self.components
    }
}

impl DerefMut for ComponentBundle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.components
    }
}
