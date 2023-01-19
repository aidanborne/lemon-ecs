use crate::{component::Component, entities::EntityId, sparse_set, world::World};

use super::{ChangeRecord, ChangeStatus};

enum ComponentSnapshot<'world, T: Component> {
    New(&'world World),
    Old(T),
    Both(&'world World, T),
}

pub struct EntitySnapshot<'world, T: Component> {
    id: EntityId,
    component: ComponentSnapshot<'world, T>,
}

impl<'world, T: Component> EntitySnapshot<'world, T> {
    pub fn get_new(&self) -> Option<&'world T> {
        match &self.component {
            ComponentSnapshot::New(world) | ComponentSnapshot::Both(world, _) => {
                world.get_component(self.id)
            }
            _ => None,
        }
    }

    pub fn get_old(&self) -> Option<&T> {
        match &self.component {
            ComponentSnapshot::Old(old) | ComponentSnapshot::Both(_, old) => Some(old),
            _ => None,
        }
    }

    #[inline]
    pub fn id(&self) -> EntityId {
        self.id
    }
}

macro_rules! downcast_vec_as_some {
    ($vec:expr) => {
        (*$vec.downcast::<Vec<T>>().ok().unwrap())
            .into_iter()
            .map(Some)
            .collect()
    };
}

pub struct SnapshotIter<'world, T: Component> {
    world: &'world World,
    iter: sparse_set::IntoIter<ChangeStatus>,
    removed: Vec<Option<T>>,
}

impl<'world, T: Component> SnapshotIter<'world, T> {
    pub(crate) fn new(world: &'world World, record: ChangeRecord) -> Self {
        Self {
            world,
            iter: record.entities.into_iter(),
            removed: downcast_vec_as_some!(record.removed),
        }
    }
}

impl<'world, T: Component> Iterator for SnapshotIter<'world, T> {
    type Item = EntitySnapshot<'world, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(id, status)| {
            let component = match status {
                ChangeStatus::Added => ComponentSnapshot::New(self.world),
                ChangeStatus::Removed(idx) => {
                    ComponentSnapshot::Old(std::mem::take(&mut self.removed[idx]).unwrap())
                }
                ChangeStatus::Modified(idx) => ComponentSnapshot::Both(
                    self.world,
                    std::mem::take(&mut self.removed[idx]).unwrap(),
                ),
            };

            EntitySnapshot { id, component }
        })
    }
}

pub struct AddedIter<'world, T: Component> {
    world: &'world World,
    iter: sparse_set::IntoIter<ChangeStatus>,
    marker: std::marker::PhantomData<T>,
}

impl<'world, T: Component> AddedIter<'world, T> {
    pub(crate) fn new(world: &'world World, record: ChangeRecord) -> Self {
        Self {
            world,
            iter: record.entities.into_iter(),
            marker: std::marker::PhantomData,
        }
    }
}

impl<'world, T: Component> Iterator for AddedIter<'world, T> {
    type Item = (EntityId, &'world T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let option = self.iter.next();

            if let Some((id, status)) = option {
                match status {
                    ChangeStatus::Added | ChangeStatus::Modified(_) => {
                        return self
                            .world
                            .get_component(id)
                            .map(|component| (id, component))
                    }
                    _ => continue,
                }
            } else {
                return None;
            }
        }
    }
}

pub struct ModifiedIter<'world, T: Component> {
    world: &'world World,
    iter: sparse_set::IntoIter<ChangeStatus>,
    removed: Vec<Option<T>>,
}

impl<'world, T: Component> ModifiedIter<'world, T> {
    pub(crate) fn new(world: &'world World, record: ChangeRecord) -> Self {
        Self {
            world,
            iter: record.entities.into_iter(),
            removed: downcast_vec_as_some!(record.removed),
        }
    }
}

impl<'world, T: Component> Iterator for ModifiedIter<'world, T> {
    type Item = (EntityId, T, &'world T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let option = self.iter.next();

            if let Some((id, status)) = option {
                match status {
                    ChangeStatus::Modified(idx) | ChangeStatus::Removed(idx) => {
                        return Some((
                            id,
                            std::mem::take(&mut self.removed[idx]).unwrap(),
                            self.world.get_component::<T>(id).unwrap(),
                        ))
                    }
                    _ => continue,
                }
            } else {
                return None;
            }
        }
    }
}

pub struct RemovedIter<T: Component> {
    iter: sparse_set::IntoIter<ChangeStatus>,
    removed: Vec<Option<T>>,
}

impl<T: Component> RemovedIter<T> {
    pub(crate) fn new(record: ChangeRecord) -> Self {
        Self {
            iter: record.entities.into_iter(),
            removed: downcast_vec_as_some!(record.removed),
        }
    }
}

impl<T: Component> Iterator for RemovedIter<T> {
    type Item = (EntityId, T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let option = self.iter.next();

            if let Some((id, status)) = option {
                match status {
                    ChangeStatus::Modified(idx) | ChangeStatus::Removed(idx) => {
                        return Some((id, std::mem::take(&mut self.removed[idx]).unwrap()))
                    }
                    _ => continue,
                }
            } else {
                return None;
            }
        }
    }
}
