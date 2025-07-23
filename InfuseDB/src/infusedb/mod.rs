// Written by Alberto Ruiz 2024-03-08
// InfuseDB is a in-memory database,
// it will store the data in memory and provide a simple API to interact with it

mod collection;
mod data_type;
pub mod utils;
pub use collection::Collection;
pub use data_type::DataType;
use std::fs;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct InfuseDB {
    pub path: String,
    collections: Vec<Collection>,
}

impl InfuseDB {
    pub fn new() -> Self {
        InfuseDB {
            path: "./default.mdb".to_string(),
            collections: Vec::new(),
        }
    }

    pub fn load(path: &str) -> Result<Self, &str> {
        let mut collections = Vec::new();
        let contents = fs::read_to_string(path);
        if contents.is_err() {
            return Err("Error reading file");
        }
        let contents = contents.unwrap();
        let mut page = String::new();
        for line in contents.lines() {
            let line = line.trim();
            let line = if line.ends_with('\n') {
                line.strip_prefix('\n').unwrap()
            } else {
                line
            };

            if line.len() == 0 || line.starts_with("#") {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') && page.len() != 0 {
                collections.push(Collection::load(page.as_str()));
                page = String::new();
            }
            page.push_str(line);
            page.push('\n');
        }
        if !page.is_empty() {
            collections.push(Collection::load(page.as_str()));
        }

        Ok(InfuseDB {
            collections,
            path: path.to_string(),
        })
    }

    pub fn dump(&self) -> Result<(), &str> {
        let mut result = String::new();
        //TODO:
        for collection in self.collections.iter() {
            let page = collection.dump();
            result.push_str(page.as_str());
            result.push_str("\n");
        }

        let _ = fs::write(self.path.as_str(), result);
        Ok(())
    }

    pub fn create_collection(&mut self, name: &str) -> Result<&mut Collection, &str> {
        //check if collection exists
        if self.collections.iter().any(|x| x.name == name) {
            Err("Collection already exists")
        } else {
            let collection = Collection::new(name);
            self.collections.push(collection);
            return Ok(self.collections.last_mut().unwrap());
        }
    }

    pub fn get_collection(&mut self, name: &str) -> Option<&mut Collection> {
        //return a mutable reference to collection
        let index = self
            .collections
            .iter()
            .position(|x| x.name == name.to_string());
        if index.is_none() {
            return None;
        }
        let index = index.unwrap();
        let c = self.collections.get_mut(index).unwrap();
        return Some(c);
    }

    pub fn get_collection_list(&self) -> Vec<String> {
        let mut collection_list: Vec<String> = Vec::new();
        for collection in self.collections.iter() {
            collection_list.push(collection.name.clone());
        }
        collection_list
    }

    pub fn remove_collection(&mut self, name: String) {
        let index = self
            .collections
            .iter()
            .position(|x| x.name == name)
            .unwrap();
        self.collections.remove(index);
    }
}

//TEST
#[cfg(test)]
#[test]
fn test_infusedb() {
    let mut infusedb = InfuseDB::new();
    let r1 = infusedb.create_collection("users").is_ok();
    let r2 = infusedb.create_collection("posts").is_ok();
    assert!(r1);
    assert!(r2);
    assert_eq!(infusedb.collections.len(), 2);
    assert_eq!(infusedb.collections[0].name, "users");
    assert_eq!(infusedb.collections[1].name, "posts");
    assert_eq!(infusedb.get_collection("users").unwrap().name, "users");
    assert_eq!(infusedb.get_collection("posts").unwrap().name, "posts");
    assert_eq!(infusedb.get_collection_list().len(), 2);
    infusedb.remove_collection("users".to_string());
    assert_eq!(infusedb.collections.len(), 1);
    infusedb.remove_collection("posts".to_string());
    assert_eq!(infusedb.collections.len(), 0);
}

// #[test]
// fn add_document() {
//     let mut infusedb = infusedb::new();
//     let _ = infusedb.create_collection("users");
//     let get_collection = infusedb.get_collection("users").unwrap();
//     let mut collection = get_collection.borrow_mut();
//     let id1 = collection.add("John", doc! {"name" => "John", "age" => 30});
//     let id2 = collection.add("Jane", doc! {"name" => "Jane", "age" => 25});
//     assert_eq!(collection.count(), 2);
//     let document = collection.get("John").unwrap();
// }

//
