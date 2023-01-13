use std::collections::HashSet;

pub struct Cacher {
    unseen: HashSet<String>,
}

impl Cacher {
    pub fn see(&mut self, s: &String) -> bool {
        self.unseen.remove(s)
    }

    pub fn all_seen(&self) -> bool {
        self.unseen.is_empty()
    }
}

impl From<Vec<String>> for Cacher {
    fn from(v: Vec<String>) -> Self {
        Self {
            unseen: v.into_iter().collect(),
        }
    }
}
