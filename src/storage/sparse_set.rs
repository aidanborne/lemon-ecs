pub struct SparseSet<T> {
    dense: Vec<(usize, T)>,
    sparse: Vec<usize>,
    len: usize,
}

#[allow(dead_code)]
impl<T> SparseSet<T> {
    pub fn new() -> Self {
        Self {
            dense: Vec::new(),
            sparse: Vec::new(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn index_of(&self, key: usize) -> Option<usize> {
        if key >= self.sparse.len() {
            return None;
        }

        let index = self.sparse[key];
        if index >= self.len || self.dense[index].0 != key {
            return None;
        }

        Some(index)
    }

    pub fn insert(&mut self, key: usize, value: T) {
        if let Some(index) = self.index_of(key) {
            self.dense[index].1 = value;
            return;
        }

        if key >= self.sparse.len() {
            self.sparse.resize(key + 1, 0);
        }

        self.sparse[key] = self.len;
        self.dense.push((key, value));
        self.len += 1;
    }

    pub fn remove(&mut self, key: usize) -> Option<T> {
        if let Some(index) = self.index_of(key) {
            let entry = self.dense.swap_remove(index);
            self.len -= 1;

            if index < self.len {
                let swapped_key = self.dense[index].0;
                self.sparse[swapped_key] = index;
            }

            Some(entry.1)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.dense.clear();
        self.sparse.clear();
        self.len = 0;
    }

    pub fn contains(&self, key: usize) -> bool {
        self.index_of(key).is_some()
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        match self.index_of(key) {
            Some(index) => Some(&self.dense[index].1),
            None => None,
        }
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        match self.index_of(key) {
            Some(index) => Some(&mut self.dense[index].1),
            None => None,
        }
    }

    pub fn get_or_insert_with(&mut self, key: usize, f: impl FnOnce() -> T) -> &mut T {
        if !self.contains(key) {
            self.insert(key, f());
        }

        self.get_mut(key).unwrap()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.dense.iter()
    }

    pub fn keys(&self) -> Keys<'_, T> {
        Keys {
            dense: self.dense.iter(),
        }
    }

    pub fn values(&self) -> Values<'_, T> {
        Values {
            dense: self.dense.iter(),
        }
    }
}

impl<T> std::ops::Index<usize> for SparseSet<T> {
    type Output = T;

    fn index(&self, key: usize) -> &Self::Output {
        self.get(key).unwrap()
    }
}

impl<T> std::ops::IndexMut<usize> for SparseSet<T> {
    fn index_mut(&mut self, key: usize) -> &mut Self::Output {
        self.get_mut(key).unwrap()
    }
}

impl<T> Default for SparseSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub type Iter<'a, T> = std::slice::Iter<'a, (usize, T)>;

pub struct Keys<'a, T> {
    dense: Iter<'a, T>,
}

impl<'a, T> Iterator for Keys<'a, T> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.dense.next().map(|entry| entry.0)
    }
}

pub struct Values<'a, T> {
    dense: Iter<'a, T>,
}

impl<'a, T> Iterator for Values<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.dense.next().map(|entry| &entry.1)
    }
}

impl<T> IntoIterator for SparseSet<T> {
    type Item = (usize, T);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.dense.into_iter()
    }
}
