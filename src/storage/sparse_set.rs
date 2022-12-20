pub struct SparseSet<T> {
    dense: Vec<Entry<T>>,
    sparse: Vec<usize>,
    len: usize,
}

pub struct Entry<T> {
    pub key: usize,
    pub value: T,
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

    pub fn dense_idx(&self, key: usize) -> Option<usize> {
        if key >= self.sparse.len() {
            return None;
        }

        let index = self.sparse[key];
        if index >= self.len || self.dense[index].key != key {
            return None;
        }

        Some(index)
    }

    pub fn insert(&mut self, key: usize, value: T) {
        if let Some(index) = self.dense_idx(key) {
            self.dense[index].value = value;
            return;
        }

        if key >= self.sparse.len() {
            self.sparse.resize(key + 1, 0);
        }

        self.sparse[key] = self.len;
        self.dense.push(Entry { key, value });
        self.len += 1;
    }

    pub fn remove(&mut self, key: usize) -> Option<T> {
        if let Some(index) = self.dense_idx(key) {
            let entry = self.dense.swap_remove(index);
            self.len -= 1;

            if index < self.len {
                let swapped_key = self.dense[index].key;
                self.sparse[swapped_key] = index;
            }

            Some(entry.value)
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
        self.dense_idx(key).is_some()
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        match self.dense_idx(key) {
            Some(index) => Some(&self.dense[index].value),
            None => None,
        }
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        match self.dense_idx(key) {
            Some(index) => Some(&mut self.dense[index].value),
            None => None,
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Entry<T>> {
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

pub type Iter<'a, T> = std::slice::Iter<'a, Entry<T>>;

pub struct Keys<'a, T> {
    dense: std::slice::Iter<'a, Entry<T>>,
}

impl<'a, T> Iterator for Keys<'a, T> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.dense.next().map(|entry| entry.key)
    }
}

pub struct Values<'a, T> {
    dense: std::slice::Iter<'a, Entry<T>>,
}

impl<'a, T> Iterator for Values<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.dense.next().map(|entry| &entry.value)
    }
}
