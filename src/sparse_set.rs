use crate::entities::EntityId;

pub(crate) struct SparseSet<T> {
    dense: Vec<(EntityId, T)>,
    sparse: Vec<usize>,
    len: usize,
}

impl<T> SparseSet<T> {
    pub fn new() -> Self {
        Self {
            dense: Vec::new(),
            sparse: Vec::new(),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            dense: Vec::with_capacity(capacity),
            sparse: Vec::with_capacity(capacity),
            len: 0,
        }
    }

    pub fn dense_idx(&self, id: EntityId) -> Option<usize> {
        if *id >= self.sparse.len() {
            return None;
        }

        let idx = self.sparse[*id];
        if idx >= self.len || self.dense[idx].0 != id {
            return None;
        }

        Some(idx)
    }

    pub fn insert(&mut self, id: EntityId, value: T) {
        if let Some(index) = self.dense_idx(id) {
            self.dense[index].1 = value;
            return;
        }

        if *id >= self.sparse.len() {
            self.sparse.resize(*id + 1, 0);
        }

        self.sparse[*id] = self.len;
        self.dense.push((id, value));
        self.len += 1;
    }

    pub fn remove(&mut self, id: EntityId) -> Option<T> {
        if let Some(index) = self.dense_idx(id) {
            let entry = self.dense.swap_remove(index);
            self.len -= 1;

            if index < self.len {
                let swapped_key = *self.dense[index].0;
                self.sparse[swapped_key] = index;
            }

            Some(entry.1)
        } else {
            None
        }
    }

    pub fn contains(&self, id: EntityId) -> bool {
        self.dense_idx(id).is_some()
    }

    /*pub fn get(&self, key: usize) -> Option<&T> {
        match self.dense_idx(key) {
            Some(index) => Some(&self.dense[index].1),
            None => None,
        }
    }*/

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut T> {
        match self.dense_idx(id) {
            Some(index) => Some(&mut self.dense[index].1),
            None => None,
        }
    }

    pub fn get_or_insert_with(&mut self, id: EntityId, f: impl FnOnce() -> T) -> &mut T {
        if !self.contains(id) {
            self.insert(id, f());
        }

        self.get_mut(id).unwrap()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.dense.iter()
    }
}

impl<T> Default for SparseSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub type Iter<'a, T> = std::slice::Iter<'a, (EntityId, T)>;

pub type IntoIter<T> = std::vec::IntoIter<(EntityId, T)>;

impl<T> IntoIterator for SparseSet<T> {
    type Item = (EntityId, T);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.dense.into_iter()
    }
}

impl<T> FromIterator<(EntityId, T)> for SparseSet<T> {
    fn from_iter<I: IntoIterator<Item = (EntityId, T)>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let size_hint = iter.size_hint();

        let mut sparse_set = Self::with_capacity(size_hint.1.unwrap_or(size_hint.0));

        for (id, value) in iter {
            sparse_set.insert(id, value);
        }

        sparse_set
    }
}
