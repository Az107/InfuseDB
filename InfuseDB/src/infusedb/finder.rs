// Written by Alberto Ruiz 2024-03-08
// [Unfinished] module to index and create search structures
//
// The B-tree will be used to index the documents
// and provide a fast search


// B-tree node
//      v: value of the content to index
//      p: position of the document in the collection
struct MIndex {
    pub v: i32,
    pub p: Vec<i32>,
}

//B-tree node
pub struct BNode {
    keys: Vec<MIndex>,
    children: Vec<Box<BNode>>,
}

impl BNode {
    pub fn new() -> BNode {
        BNode {
            keys: Vec::new(),
            children: Vec::new(),
        }
    }

    fn _search(node: &BNode, value: i32) -> Option<Vec<i32>> {
        let mut i = 0;
        while i < node.keys.len() && value > node.keys[i].v {
            i += 1;
        }
        if i < node.keys.len() && value == node.keys[i].v {
            return Some(node.keys[i].p.clone());
        }
        if node.children.len() > 0 {
            return BNode::_search(&node.children[i], value);
        }
        None
    }

    pub fn search(&self, value: i32) -> Option<Vec<i32>> {
        BNode::_search(&self, value)
    }

    pub fn insert(&mut self, value: i32, pos: i32) {
        let mut i = 0;
        while i < self.keys.len() && value > self.keys[i].v {
            i += 1;
        }
        if i < self.keys.len() && value == self.keys[i].v {
            self.keys[i].p.push(pos);
            return;
        }
        if self.children.len() > 0 {
            self.children[i].insert(value, pos);
            return;
        }
        self.keys.insert(i, MIndex { v: value, p: vec![pos] });
    }


}