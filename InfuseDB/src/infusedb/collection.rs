use crate::d;
use crate::utils;
// Writen by Alberto Ruiz 2024-03-08
// The collection module will provide the collection of documents for the InfuseDB
// The collection will store the documents in memory and provide a simple API to interact with them
// The Document will be a HashMap<String, DataType>
//
use super::data_type::DataType;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

const ID: &str = "ID";

//create a trait based on HashMap<String,DataType>
// and impl especial methods for it
pub type Document = HashMap<String, DataType>;

// impl Into<DataType> for Document {
//     fn into(self) -> DataType {
//         DataType::Document(self)
//     }
// }

pub trait DocumentJson {
    fn to_json(&self) -> String;
    fn to_json_value(&self) -> Value;
    fn from_json(json: &str) -> Result<Document, &str>;
}

impl DocumentJson for Document {
    fn to_json(&self) -> String {
        self.to_json_value().to_string()
    }

    fn to_json_value(&self) -> Value {
        let mut json = serde_json::json!({});
        for (key, value) in self.iter() {
            match value {
                DataType::Id(id) => json[key] = serde_json::json!(id.to_string()),
                DataType::Text(text) => json[key] = serde_json::json!(text),
                DataType::Number(number) => json[key] = serde_json::json!(number),
                DataType::Boolean(boolean) => json[key] = serde_json::json!(boolean),
                _ => json[key] = serde_json::json!("()"),
            }
        }
        json
    }

    fn from_json(json: &str) -> Result<Self, &str> {
        let v: Value = serde_json::from_str(json).unwrap();
        let mut document = Document::new();
        for (key, value) in v.as_object().unwrap() {
            let value: Value = value.clone();
            if key == ID {
                let value_is_string = value.is_string();
                let id = Uuid::parse_str(value.as_str().unwrap());
                if id.is_ok() && value_is_string {
                    document.insert(key.to_string(), DataType::Id(id.unwrap()));
                } else {
                    match value {
                        Value::Number(n) => document.insert(
                            "id".to_string(),
                            DataType::Number(n.as_i64().unwrap() as i32),
                        ),
                        Value::String(s) => document.insert("id".to_string(), DataType::Text(s)),
                        Value::Bool(b) => document.insert("id".to_string(), DataType::Boolean(b)),
                        _ => document.insert("id".to_string(), DataType::Text("".to_string())),
                    };
                }
            } else {
                match value {
                    Value::Number(n) => document.insert(
                        key.to_string(),
                        DataType::Number(n.as_i64().unwrap() as i32),
                    ),
                    Value::String(s) => document.insert(key.to_string(), DataType::Text(s)),
                    Value::Bool(b) => document.insert(key.to_string(), DataType::Boolean(b)),
                    _ => document.insert(key.to_string(), DataType::Text("".to_string())),
                };
            }
        }

        Ok(document)
    }
}

//create a macro to create a document
#[macro_export]
macro_rules! doc {
  ( $( $key: expr => $value: expr ),* ) => {
    {
         use std::collections::HashMap;
        let mut map = HashMap::new();
        $(
            map.insert($key.to_string(), DataType::from($value)); // Update this line
        )*
        DataType::Document(map)
    }
  };
}

pub struct Collection {
    pub name: String,
    pub(crate) data: HashMap<String, DataType>,
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

// impl DocumentJson for Collection {
//     fn to_json_value(&self) -> Value {
//         let mut json = serde_json::json!({
//           "name": self.name,
//           "data": []
//         });
//         let mut jsondata = Vec::new();
//         for document in self.data.iter() {
//             jsondata.push(document.to_json_value())
//         }
//         json["data"] = serde_json::Value::Array(jsondata);
//         json
//     }

//     fn to_json(&self) -> String {
//         self.to_json_value().to_string()
//     }

//     fn from_json(json: &str) -> Result<Self, &str> {
//         let v: Value = serde_json::from_str(json).unwrap();
//         let obj = v.as_object().unwrap();
//         let name = obj.get("name");
//         if name.is_none() {
//             return Err("()");
//         }
//         let name = name.unwrap().to_string().replace("\"", "");
//         let mut collection = Collection::new(name.as_str());
//         let data = obj.get("data");
//         if data.is_none() {
//             return Err("()");
//         }
//         let data = data.unwrap().as_array();
//         if data.is_none() {
//             return Err("Error converting data as array");
//         }
//         let data = data.unwrap();
//         for document in data {
//             let document = document.to_string();
//             let document = document.as_str();
//             let doc = Document::from_json(document);
//             if doc.is_err() {
//                 continue;
//             }
//             let doc = doc.unwrap();
//             collection.add(doc);
//         }
//         return Ok(collection);
//     }
// }

// impl KV for Collection {
impl Collection {
    pub fn new(name: &str) -> Self {
        Collection {
            name: name.to_string(),
            data: HashMap::new(), //b_tree: BNode::new(),
        }
    }

    pub fn add(&mut self, key: &str, value: DataType) -> &mut Self {
        self.data.insert(key.to_string(), value);
        return self;
    }

    pub fn add_subkey(&mut self, key: &str, subkey: &str, value: DataType) -> &mut Self {
        if let Some(dt) = self.data.get_mut(key) {
            let _ = dt.set(subkey, value); //TODO: better error handling
        } else {
            if let Ok(index) = subkey.parse::<usize>() {
                //Array
                let mut v = Vec::new();
                v[index] = value;
                let dt = DataType::Array(v);
                self.data.insert(key.to_string(), dt);
            } else {
                //Object
                let mut d = Document::new();
                d.insert(subkey.to_string(), value);
                self.data.insert(key.to_string(), DataType::Document(d));
            }
        }
        return self;
    }

    pub fn rm(&mut self, key: &str) {
        //self.data.remove(index);
        self.data.remove(key);
    }

    pub fn count(&self) -> usize {
        self.data.len()
    }

    pub fn list(&self) -> HashMap<String, DataType> {
        return self.data.clone();
    }

    pub fn get(&mut self, key: &str) -> Option<&DataType> {
        return self.data.get(key);
    }

    pub fn dump(&self) -> String {
        let mut result = String::new();
        result.push_str(format!("[{}]\n", self.name).as_str());
        for (k, v) in self.data.iter() {
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
    collection.add("name", d!("Juan"));
    collection.add("surname", d!("Perez"));
    collection.add("age", d!(15));
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
