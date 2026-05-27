use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    ptr::null_mut,
};

pub struct LRU<T> {
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
    pin: usize,
}

impl<T> Node<T> {
    pub fn new(key: u32, data: T) -> Self {
        Node {
            next: null_mut(),
            prev: null_mut(),
            data,
            key,
            pin: 0,
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
        let ptr = *self.map.get(&id)?;
        self.move_to_head(ptr);
        unsafe { Some(&(*ptr).data) }
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut T> {
        let ptr = *self.map.get(&id)?;
        self.move_to_head(ptr);
        unsafe { Some(&mut (*ptr).data) }
    }

    pub fn pin(&mut self, id: u32) -> Result<(), Error> {
        let ptr = *self
            .map
            .get(&id)
            .ok_or(Error::new(ErrorKind::NotFound, "Not found"))?;
        unsafe {
            (*ptr).pin += 1;
        }
        Ok(())
    }

    pub fn unpin(&mut self, id: u32) -> Result<(), Error> {
        let ptr = *self
            .map
            .get(&id)
            .ok_or(Error::new(ErrorKind::NotFound, "Not found"))?;
        unsafe {
            if (*ptr).pin == 0 {
                return Err(Error::new(ErrorKind::InvalidData, "Not pinned element"));
            }
            (*ptr).pin -= 1;
        }
        Ok(())
    }

    pub fn get_evict_candidate(&self) -> Option<u32> {
        let mut candidate = self.tail;
        unsafe {
            while !candidate.is_null() && (*candidate).pin == 0 {
                candidate = (*candidate).prev
            }
            if candidate.is_null() {
                None
            } else {
                Some((*candidate).key)
            }
        }
    }

    pub fn write<F, R>(&mut self, id: u32, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        let ptr = *self.map.get(&id)?;
        self.move_to_head(ptr);
        unsafe {
            (*ptr).pin += 1;
            let result = f(&mut (*ptr).data);
            (*ptr).pin -= 1;
            Some(result)
        }
    }

    pub fn evict(&mut self) -> Result<(), Error> {
        if self.tail.is_null() {
            return Err(Error::new(ErrorKind::AddrNotAvailable, "Tail is Null"));
        }
        let mut candidate = self.tail;
        unsafe {
            while !candidate.is_null() {
                if (*candidate).pin == 0 {
                    break;
                }
                candidate = (*candidate).prev;
            }
            if candidate.is_null() {
                return Err(Error::new(
                    ErrorKind::OutOfMemory,
                    "Not evictable elements found",
                ));
            }
            self.detach(candidate);
            self.count -= 1;
            self.map.remove(&(*candidate).key);
            drop(Box::from_raw(candidate));
        }
        Ok(())
    }

    fn detach(&mut self, node: *mut Node<T>) {
        unsafe {
            if (*node).prev.is_null() {
                // Is head
                self.head = (*node).next;
                if !self.head.is_null() {
                    (*self.head).prev = null_mut();
                } else {
                    //only item
                    self.tail = null_mut();
                }
            } else if (*node).next.is_null() {
                //Is tail
                self.tail = (*node).prev;
                if !self.tail.is_null() {
                    (*self.tail).next = null_mut();
                } else {
                    //only item
                    self.head = null_mut();
                }
            } else {
                (*(*node).prev).next = (*node).next;
                (*(*node).next).prev = (*node).prev;
            }
        }
    }

    fn move_to_head(&mut self, node: *mut Node<T>) {
        unsafe {
            if node.is_null() || self.head == node {
                return;
            }

            if !(*node).prev.is_null() || !(*node).next.is_null() {
                self.detach(node);
            }

            (*node).prev = null_mut();
            (*node).next = self.head;
            if !self.head.is_null() {
                (*self.head).prev = node;
            }
            self.head = node;
            if self.tail.is_null() {
                self.tail = self.head
            }
        }
    }

    pub fn set(&mut self, key: u32, data: T) -> Result<(), Error> {
        if self.count >= self.max_nodes {
            self.evict()?;
        }
        let node = Node::new(key, data);
        let ptr = Box::into_raw(Box::new(node));
        self.move_to_head(ptr);
        self.map.insert(key, ptr);
        self.count += 1;
        Ok(())
    }

    pub fn has(&mut self, key: u32) -> bool {
        self.map.contains_key(&key)
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

    #[test]
    fn test_eviction() {
        let mut cache = LRU::<String>::new(2);
        let payload = "Hello World".to_string();
        assert_eq!(cache.count, 0);
        let r = cache.set(0, payload.clone());
        let r = cache.set(1, payload.clone());
        let r = cache.set(2, payload.clone());
        assert!(r.is_ok());
        assert_eq!(cache.count, 2);
        assert!(!cache.head.is_null());
        assert!(!cache.tail.is_null());
        unsafe {
            assert_eq!((*cache.head).key, 2);
            assert_eq!((*cache.tail).key, 1);
        }
    }

    #[test]
    fn test_eviction_pin() {
        let mut cache = LRU::<String>::new(2);
        let payload = "Hello World".to_string();
        assert_eq!(cache.count, 0);
        let _ = cache.set(0, payload.clone());
        let _ = cache.set(1, payload.clone());
        let r = cache.pin(0);
        assert!(r.is_ok());
        let _ = cache.set(2, payload.clone());

        assert_eq!(cache.count, 2);
        assert!(!cache.head.is_null());
        assert!(!cache.tail.is_null());
        unsafe {
            assert_eq!((*cache.head).key, 2);
            assert_eq!((*cache.tail).key, 0);
        }
    }

    #[test]
    fn test_eviction_pin_all() {
        let mut cache = LRU::<String>::new(2);
        let payload = "Hello World".to_string();
        assert_eq!(cache.count, 0);
        let _ = cache.set(0, payload.clone());
        let _ = cache.set(1, payload.clone());
        let r = cache.pin(0);
        assert!(r.is_ok());
        let r = cache.pin(1);
        assert!(r.is_ok());
        let r = cache.set(2, payload.clone());
        assert!(r.is_err());
        assert_eq!(cache.count, 2);
        assert!(!cache.head.is_null());
        assert!(!cache.tail.is_null());
        unsafe {
            assert_eq!((*cache.head).key, 1);
            assert_eq!((*cache.tail).key, 0);
        }
    }

    #[test]
    fn test_eviction_unpin() {
        let mut cache = LRU::<String>::new(2);
        let payload = "Hello World".to_string();
        assert_eq!(cache.count, 0);
        let _ = cache.set(0, payload.clone());
        let _ = cache.set(1, payload.clone());
        let _ = cache.pin(0);
        let _ = cache.pin(1);
        let r = cache.unpin(1);
        assert!(r.is_ok());
        let r = cache.set(2, payload.clone());
        assert!(r.is_ok());
        assert_eq!(cache.count, 2);
        assert!(!cache.head.is_null());
        assert!(!cache.tail.is_null());
        unsafe {
            assert_eq!((*cache.head).key, 2);
            assert_eq!((*cache.tail).key, 0);
        }
    }
}
