use std::{collections::HashMap, ptr::null_mut};

struct LRU<T> {
    max_nodes: u32,
    count: u32,
    head: *mut Node<T>,
    tail: *mut Node<T>,
    map: HashMap<u32, *mut Node<T>>,
}

struct Node<T> {
    next: *mut Node<T>,
    prev: *mut Node<T>,
    data: T,
    key: u32,
}

impl<T> Node<T> {
    pub fn new(key: u32, data: T) -> Self {
        Node {
            next: null_mut(),
            prev: null_mut(),
            data,
            key,
        }
    }
}

impl<T> LRU<T> {
    pub fn new(max_nodes: u32) -> Self {
        LRU {
            max_nodes,
            count: 0,
            map: HashMap::new(),
            head: null_mut(),
            tail: null_mut(),
        }
    }

    pub fn get(&mut self, id: u32) -> Option<&T> {
        let ptr = self.map.get(&id)?.clone();
        self.move_to_head(ptr);
        unsafe {
            if !ptr.is_null() {
                Some(&(*ptr).data)
            } else {
                None
            }
        }
    }

    fn evict(&mut self) {
        todo!()
    }

    fn move_to_head(&mut self, node: *mut Node<T>) {
        unsafe {
            if node.is_null() || self.head == node {
                return;
            }

            if !(*node).prev.is_null() {
                (*(*node).prev).next = (*node).next
            }

            if !(*node).next.is_null() {
                (*(*node).next).prev = (*node).prev;
            } else {
                // Node is Tail
                self.tail = if self.tail.is_null() {
                    node
                } else if !(*node).prev.is_null() {
                    (*node).prev
                } else {
                    self.tail
                }
            }

            (*node).prev = null_mut();
            (*node).next = self.head;
            if !self.head.is_null() {
                (*self.head).prev = node;
            }
            self.head = node;
        }
    }

    pub fn set(&mut self, key: u32, data: T) -> Result<(), &'static str> {
        if self.count >= self.max_nodes {
            self.evict();
        }
        let node = Node::new(key, data);
        let ptr = Box::into_raw(Box::new(node));
        self.move_to_head(ptr);
        self.map.insert(key, ptr);
        self.count += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_empty_cache() {
        let mut cache = LRU::<String>::new(10);
        let payload = "Hello World".to_string();
        assert_eq!(cache.count, 0);
        let r = cache.set(0, payload.clone());
        assert_eq!(cache.count, 1);
        assert!(r.is_ok());
        let r = cache.get(0);
        assert!(r.is_some());
        let r = r.unwrap();
        assert_eq!(r, &payload);
        assert_eq!(cache.head, cache.tail);
    }

    #[test]
    fn test_cache() {
        let mut cache = LRU::<String>::new(10);
        let payload = "Hello World".to_string();
        assert_eq!(cache.count, 0);
        let r = cache.set(0, payload.clone());
        assert!(r.is_ok());
        assert_eq!(cache.count, 1);
        let r = cache.set(1, payload.clone());
        assert!(r.is_ok());
        assert_eq!(cache.count, 2);

        assert!(!cache.head.is_null());
        assert!(!cache.tail.is_null());
        unsafe {
            assert_eq!((*cache.head).key, 1);
            assert_eq!((*cache.tail).key, 0);
        }

        let r = cache.get(0);
        assert!(r.is_some());
        let r = r.unwrap();
        assert_eq!(r, &payload);
        assert_ne!(cache.head, cache.tail);
        unsafe {
            assert_eq!((*cache.head).key, 0);
            assert_eq!((*cache.tail).key, 1);
        }
    }
}
