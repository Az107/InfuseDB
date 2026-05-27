// Writen by Alberto Ruiz 2024-03-08
// The collection module will provide the collection of documents for the InfuseDB
// The collection will store the documents in memory and provide a simple API to interact with them
// The Document will be a HashMap<String, DataType>
//
use super::data_type::DataType;
use super::{
    file_engine::buffer_pool::BufferPool,
    translator::{PageHandler, expand_pointer},
    utils,
};

use std::{cell::RefCell, collections::HashMap, io::Error, rc::Rc};

pub type Document = HashMap<String, DataType>;

#[macro_export]
macro_rules! doc {
  ( $( $key: expr => $value: expr ),* ) => {
    {
         use std::collections::HashMap;
        let mut map = HashMap::new();
        $(
            map.insert($key.to_string(), DataType::from($value));
        )*
        DataType::Document(map)
    }
  };
}

pub struct Collection {
    pub name: String,
    index: PageHandler,
    data: PageHandler,
    buffer_pool: Rc<RefCell<BufferPool>>,
}

pub trait _KV {
    fn new(name: &str) -> Self;
    fn add(&mut self, key: &str, value: DataType) -> &mut Self;
    fn rm(&mut self, key: &str);
    fn count(&self) -> usize;
    fn list(&self) -> HashMap<String, DataType>;
    fn get(&mut self, key: &str) -> Option<&DataType>;
    fn dump(&self) -> String;
    fn load(data: &str) -> Collection;
}

// impl KV for Collection {
impl Collection {
    pub fn new(name: &str, index_id: u32, buffer_pool: Rc<RefCell<BufferPool>>) -> Self {
        let index_handler = PageHandler::new(buffer_pool.clone(), index_id);
        let data_id = index_handler.get("__data").unwrap().to_pointer();
        let data = PageHandler::new(buffer_pool.clone(), data_id.0);

        Collection {
            name: name.to_string(),
            buffer_pool,
            index: index_handler,
            data,
        }
    }

    pub fn add(&mut self, key: &str, value: DataType) -> Result<(), Error> {
        let pointer = self.data.set(key, value)?;
        self.index
            .set(key, DataType::Pointer(pointer.0, pointer.1))?;
        Ok(())
    }

    pub fn rm(&mut self, key: &str) {
        todo!();
    }

    pub fn count(&self) -> usize {
        todo!()
    }

    pub fn list(&self) -> HashMap<String, DataType> {
        let mut list = HashMap::new();
        for item in self.index.list() {
            list.insert(item, DataType::Text("niputaidea".to_string()));
        }
        list
    }

    pub fn get(&mut self, key: &str) -> Option<DataType> {
        let pointer = self.index.get(key)?.to_pointer();
        expand_pointer(&mut self.buffer_pool.borrow_mut(), pointer.0, pointer.1).ok()
    }

    pub fn dump(&self) -> String {
        todo!()
    }
}

//TEST
// #[cfg(test)]
// #[test]
// fn test_collection() {
//     let mut collection = Collection::new("users");
//     collection.add(
//         "John",
//         doc!(
//           "name" => "John",
//           "age" => 25,
//           "isMarried" => false,
//           "birthDate" => "1995-01-01"
//         ),
//     );
//     assert!(collection.get("John").is_some());
// }

// #[test]
// fn test_dump() {
//     let header = "[prueba]\n";
//     let kv_name = "2 name \"Juan\"";
//     let kv_surname = "2 surname \"Perez\"";
//     let kv_age = "3 age 15";

//     let mut collection = Collection::new("prueba");
//     collection.add("name", DataType::from("Juan"));
//     collection.add("surname", DataType::from("Perez"));
//     collection.add("age", DataType::from(15));

//     let dump = collection.dump();
//     println!("{}", dump);
//     assert!(dump.starts_with(header));
//     assert!(dump.contains(kv_name));
//     assert!(dump.contains(kv_surname));
//     assert!(dump.contains(kv_age));
// }

// #[test]
// fn test_load() {
//     let dump = "[prueba]\n2 name Juan\n2 surname Perez\n3 age 15\n";
//     let c = Collection::load(dump);
//     assert_eq!(c.name, "prueba");
// }
