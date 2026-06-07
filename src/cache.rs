use std::collections::{HashMap, VecDeque};

pub struct Cache {
    map: HashMap<String, String>,
    order: VecDeque<String>,
    capacity: usize,
}

impl Cache {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&str> {
        if self.map.contains_key(key) {
            // Move to back (most recently used)
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                if let Some(k) = self.order.remove(pos) {
                    self.order.push_back(k);
                }
            }
            self.map.get(key).map(|s| s.as_str())
        } else {
            None
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        if self.map.contains_key(&key) {
            // Update existing
            self.map.insert(key.clone(), value);
            if let Some(pos) = self.order.iter().position(|k| *k == key) {
                if let Some(k) = self.order.remove(pos) {
                    self.order.push_back(k);
                }
            }
            return;
        }

        if self.map.len() >= self.capacity {
            self.evict_one();
        }

        self.order.push_back(key.clone());
        self.map.insert(key, value);
    }

    fn evict_one(&mut self) {
        while let Some(k) = self.order.pop_front() {
            if self.map.remove(&k).is_some() {
                break;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ops() {
        let mut c = Cache::new(3);
        c.set("a".into(), "1".into());
        c.set("b".into(), "2".into());
        c.set("c".into(), "3".into());
        assert_eq!(c.get("a"), Some("1"));
        assert_eq!(c.get("b"), Some("2"));
        assert_eq!(c.get("c"), Some("3"));
        assert_eq!(c.get("d"), None);
    }

    #[test]
    fn test_eviction() {
        let mut c = Cache::new(2);
        c.set("a".into(), "1".into());
        c.set("b".into(), "2".into());
        c.set("c".into(), "3".into());
        // 'a' should be evicted
        assert_eq!(c.get("a"), None);
        assert_eq!(c.get("b"), Some("2"));
        assert_eq!(c.get("c"), Some("3"));
    }

    #[test]
    fn test_lru_promotion() {
        let mut c = Cache::new(2);
        c.set("a".into(), "1".into());
        c.set("b".into(), "2".into());
        c.get("a"); // promotes 'a' to MRU
        c.set("c".into(), "3".into());
        // 'b' should be evicted, 'a' still present
        assert_eq!(c.get("a"), Some("1"));
        assert_eq!(c.get("b"), None);
        assert_eq!(c.get("c"), Some("3"));
    }
}
