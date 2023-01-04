use std::{cell::Cell, ops::Deref};

#[derive(Default)]
pub struct Entities {
    available_ids: Vec<usize>,
    next_id: Cell<usize>,
}

impl Entities {
    pub fn new() -> Self {
        Self {
            available_ids: Vec::new(),
            next_id: Cell::new(0),
        }
    }

    pub fn spawn(&mut self) -> usize {
        let id = match self.available_ids.pop() {
            Some(id) => id,
            None => {
                let id = self.next_id.get();
                self.next_id.set(id + 1);
                id
            }
        };
        id
    }

    pub fn despawn(&mut self, id: usize) {
        self.available_ids.push(id);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct EntityId(usize);

impl EntityId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}

impl From<usize> for EntityId {
    fn from(id: usize) -> Self {
        Self::new(id)
    }
}

impl Deref for EntityId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
