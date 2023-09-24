use std::collections::HashSet;
use std::hash::Hash;

pub struct Cacher<T> {
    unseen: Option<HashSet<T>>,
}

impl<T: Eq + PartialEq + Hash> Cacher<T> {
    pub fn has_cached(&mut self, s: &T) -> bool {
        !self.unseen.as_mut().map(|hs| hs.remove(s)).unwrap_or(true)
    }

    pub fn is_done(&self) -> bool {
        self.unseen
            .as_ref()
            .map(|hs| hs.is_empty())
            .unwrap_or(false)
    }
}

impl<T: Eq + PartialEq + Hash> From<Option<Vec<T>>> for Cacher<T> {
    fn from(opt_v: Option<Vec<T>>) -> Self {
        Self {
            unseen: opt_v.map(|v| v.into_iter().collect()),
        }
    }
}
