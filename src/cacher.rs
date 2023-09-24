use std::collections::HashSet;
use std::hash::Hash;

pub struct Cacher<T> {
    unseen: HashSet<T>,
}

impl<T: Eq+PartialEq+Hash> Cacher<T> {
    pub fn see(&mut self, s: &T) -> bool {
        self.unseen.remove(s)
    }

    pub fn all_seen(&self) -> bool {
        self.unseen.is_empty()
    }
}

impl<T: Eq+PartialEq+Hash> From<Vec<T>> for Cacher<T> {
    fn from(v: Vec<T>) -> Self {
        Self {
            unseen: v.into_iter().collect(),
        }
    }
}
