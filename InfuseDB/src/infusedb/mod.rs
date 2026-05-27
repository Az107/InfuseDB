// Written by Alberto Ruiz 2024-03-08
// InfuseDB is a in-memory database,
// it will store the data in memory and provide a simple API to interact with it

mod collection;
mod data_type;
mod file_engine;
mod translator;
pub mod utils;
pub use collection::Collection;
pub use data_type::DataType;
pub use data_type::FindOp;
use std::cell::RefCell;
//TODO: change to own trait and file
use std::io::Error;

use std::rc::Rc;

use file_engine::buffer_pool::BufferPool;

use translator::PageHandler;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct InfuseDB {
    pub path: String,
    buffer_pool: Rc<RefCell<BufferPool>>,
    index_id: u32,
    index: PageHandler,
}

impl InfuseDB {
    pub fn new(path: &str) -> Self {
        let buffer_pool = Rc::new(RefCell::new(BufferPool::new(path).unwrap()));
        if buffer_pool.borrow_mut().get_page(1).is_none() {
            buffer_pool
                .borrow_mut()
                .create_page(file_engine::PageType::Index);
        }
        let index = PageHandler::new(buffer_pool.clone(), 1);
        InfuseDB {
            path: path.to_string(),
            buffer_pool,
            index_id: 1,
            index,
        }
    }

    pub fn create_collection(&mut self, name: &str) -> Result<Collection, Error> {
        let (index_id, data_id) = {
            let mut bp = self.buffer_pool.borrow_mut();
            let index_id = bp.create_page(file_engine::PageType::Index)?;
            let data_id = bp.create_page(file_engine::PageType::Data)?;
            (index_id, data_id)
        };

        self.index.set(name, DataType::Pointer(index_id, 0xffff))?;
        PageHandler::new(self.buffer_pool.clone(), index_id)
            .set("__data", DataType::Pointer(data_id, 0xffff))?;

        Ok(Collection::new(name, index_id, self.buffer_pool.clone()))
    }

    pub fn get_collection(&mut self, name: &str) -> Option<Collection> {
        //return a mutable reference to collection
        let ptr = self.index.get(name)?.to_pointer();
        let collection = Collection::new(name, ptr.0, Rc::clone(&self.buffer_pool));
        Some(collection)
    }

    pub fn get_collection_list(&self) -> Vec<String> {
        self.index.list()
    }

    pub fn remove_collection(&mut self, name: String) {
        todo!()
    }

    pub fn commit(&self) {
        self.buffer_pool
            .borrow_mut()
            .flush_all()
            .expect("Error flushing all to the disk");
    }
}

//TEST
// #[cfg(test)]
// #[test]
// fn test_infusedb() {
//     let mut infusedb = InfuseDB::new();
//     let r1 = infusedb.create_collection("users").is_ok();
//     let r2 = infusedb.create_collection("posts").is_ok();
//     assert!(r1);
//     assert!(r2);
//     assert_eq!(infusedb.collections.len(), 2);
//     assert_eq!(infusedb.collections[0].name, "users");
//     assert_eq!(infusedb.collections[1].name, "posts");
//     assert_eq!(infusedb.get_collection("users").unwrap().name, "users");
//     assert_eq!(infusedb.get_collection("posts").unwrap().name, "posts");
//     assert_eq!(infusedb.get_collection_list().len(), 2);
//     infusedb.remove_collection("users".to_string());
//     assert_eq!(infusedb.collections.len(), 1);
//     infusedb.remove_collection("posts".to_string());
//     assert_eq!(infusedb.collections.len(), 0);
// }

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
