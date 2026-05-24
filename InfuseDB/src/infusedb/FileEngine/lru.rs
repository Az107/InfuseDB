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

    pub fn get(&mut self, id: u32) -> Option<T> {
        None
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
                (*(*node).prev).next = (*node).next;
            }

            if !(*node).next.is_null() {
                (*(*node).next).prev = (*node).prev;
            } else {
                self.tail = (*node).prev;
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
        let mut ptr = Box::into_raw(Box::new(node));
        self.move_to_head(ptr);
        self.map.insert(key, ptr);
        Ok(())
    }
}
