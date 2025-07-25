// Writen by Alberto Ruiz 2024-03-08
// The collection module will provide the collection of documents for the InfuseDB
// The collection will store the documents in memory and provide a simple API to interact with them
// The Document will be a HashMap<String, DataType>
//
use super::data_type::DataType;
use crate::utils;
use std::collections::HashMap;

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
    pub(crate) data: DataType,
    //b_tree: BNode
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
    pub fn new(name: &str) -> Self {
        Collection {
            name: name.to_string(),
            data: DataType::Document(Document::new()),
            //b_tree: BNode::new(),
        }
    }

    pub fn add(&mut self, key: &str, value: DataType) -> &mut Self {
        let _ = self.data.set(key, value);
        return self;
    }

    pub fn rm(&mut self, key: &str) {
        let _ = self.data.remove(key);
    }

    pub fn count(&self) -> usize {
        self.data.to_document().len()
    }

    pub fn list(&self) -> HashMap<String, DataType> {
        return self.data.to_document().clone();
    }

    pub fn get(&mut self, key: &str) -> Option<&DataType> {
        return self.data.get(key);
    }

    pub fn dump(&self) -> String {
        let mut result = String::new();
        result.push_str(format!("[{}]\n", self.name).as_str());
        for (k, v) in self.data.to_document().iter() {
            let t = match v.get_type() {
                "id" => "1",
                "text" => "2",
                "number" => "3",
                "boolean" => "4",
                "array" => "5",
                "document" => "6",
                _ => "7",
            };
            let line = format!("{} {} {}\n", t, k, v.to_string());
            result.push_str(line.as_str());
        }
        return result;
    }

    pub fn load(data: &str) -> Collection {
        let data_text = data.to_string();
        let parser = data_text.lines();
        let name = parser.clone().next();
        if name.is_none() {
            panic!("invalid data")
        }
        let name = name
            .unwrap()
            .strip_suffix(']')
            .unwrap()
            .strip_prefix('[')
            .unwrap();
        let mut result = Collection::new(name);
        for line in parser.into_iter() {
            if line.starts_with('[') {
                continue;
            }
            let line_text = line.to_string();
            let elements = utils::smart_split(line_text);
            if elements.len() != 3 {
                continue;
            }
            let raw_t = elements[0].clone();
            let t = raw_t.parse::<u16>();
            if t.is_err() {
                println!("Error parsing: {:?}", t.err());
                continue;
            }
            let t = t.unwrap();
            let k = elements[1].clone();
            let raw_v = elements[2].clone();
            let v = DataType::load(t, raw_v);
            if v.is_none() {
                println!("Error parsing: unresolved value");
                continue;
            }
            let v = v.unwrap();
            result.add(k.as_str(), v);
        }

        return result;
    }
}

//TEST
#[cfg(test)]
#[test]
fn test_collection() {
    let mut collection = Collection::new("users");
    collection.add(
        "John",
        doc!(
          "name" => "John",
          "age" => 25,
          "isMarried" => false,
          "birthDate" => "1995-01-01"
        ),
    );
    assert!(collection.get("John").is_some());
}

#[test]
fn test_dump() {
    let header = "[prueba]\n";
    let kv_name = "2 name \"Juan\"";
    let kv_surname = "2 surname \"Perez\"";
    let kv_age = "3 age 15";

    let mut collection = Collection::new("prueba");
    collection.add("name", DataType::from("Juan"));
    collection.add("surname", DataType::from("Perez"));
    collection.add("age", DataType::from(15));

    let dump = collection.dump();
    println!("{}", dump);
    assert!(dump.starts_with(header));
    assert!(dump.contains(kv_name));
    assert!(dump.contains(kv_surname));
    assert!(dump.contains(kv_age));
}

#[test]
fn test_load() {
    let dump = "[prueba]\n2 name Juan\n2 surname Perez\n3 age 15\n";
    let c = Collection::load(dump);
    assert_eq!(c.name, "prueba");
}
