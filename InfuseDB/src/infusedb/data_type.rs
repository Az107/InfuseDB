// Written by Alberto Ruiz 2024-03-08
// The data type module will provide the data types for the InfuseDB
// this will be store several types of data, like text, numbers, dates, arrays and documents
//
// The data type will be used to store the data in the documents
use super::collection::Document;
use uuid::Uuid;

#[derive(PartialEq, Debug)]
pub enum DataType {
    Id(Uuid),
    Text(String),
    Number(i32),
    Boolean(bool),
    Array(Vec<DataType>),
    Document(Document),
}

impl DataType {
    pub fn get_type(&self) -> &str {
        match self {
            DataType::Id(_) => "id",
            DataType::Text(_) => "text",
            DataType::Number(_) => "number",
            DataType::Boolean(_) => "boolean",
            DataType::Array(_) => "array",
            DataType::Document(_) => "document",
        }
    }

    pub fn get(&self, n: usize) -> DataType {
        //WIP 🚧
        if matches!(self, DataType::Array(_)) {
            return self.to_array().get(n).unwrap().clone();
        } else {
            return self.clone();
        }
    }

    pub fn set(&mut self, index: &str, dt: DataType) -> Result<DataType, &'static str> {
        match self {
            DataType::Array(vec) => {
                if let Ok(index) = index.parse::<usize>() {
                    while index > vec.len() - 1 {
                        vec.push(DataType::Text("".to_string()));
                    }
                    vec[index] = dt;
                    Ok(self.clone())
                } else {
                    Err("Invalid index")
                }
            }
            DataType::Document(doc) => {
                doc.insert(index.to_string(), dt);
                Ok(self.clone())
            }
            _ => Err("Not supported"),
        }
    }

    pub fn concat(&self, b: DataType) -> Option<Self> {
        if !matches!(self, DataType::Array(_)) && b.get_type() != self.get_type() {
            return None;
        }
        let result;
        match self {
            DataType::Text(text) => {
                let mut new_text = text.clone();
                new_text.push_str(b.to_text());
                result = DataType::Text(new_text.clone());
            }
            DataType::Number(num) => {
                let new_num = num + b.to_number();
                result = DataType::Number(new_num)
            }
            DataType::Array(list) => {
                let mut new_list = list.clone();
                new_list.push(b);
                result = DataType::Array(new_list);
            }
            _ => return None,
        };
        return Some(result);
    }

    //add into
    pub fn to_id(&self) -> Uuid {
        match self {
            DataType::Id(id) => *id,
            _ => panic!("Not an ID"),
        }
    }
    pub fn to_text(&self) -> &String {
        match self {
            DataType::Text(text) => text,
            _ => panic!("Not a Text"),
        }
    }
    pub fn to_number(&self) -> i32 {
        match self {
            DataType::Number(number) => *number,
            _ => panic!("Not a Number"),
        }
    }
    pub fn to_boolean(&self) -> bool {
        match self {
            DataType::Boolean(boolean) => *boolean,
            _ => panic!("Not a Boolean"),
        }
    }
    pub fn to_array(&self) -> &Vec<DataType> {
        match self {
            DataType::Array(array) => array,
            _ => panic!("Not an Array"),
        }
    }
    pub fn to_document(&self) -> &Document {
        match self {
            DataType::Document(document) => document,
            _ => panic!("Not a Document"),
        }
    }

    pub fn infer_type(raw: &str) -> u16 {
        let raw = raw.trim();
        if Uuid::parse_str(raw).is_ok() {
            1
        } else if raw.parse::<i32>().is_ok() {
            3
        } else if raw.to_lowercase().as_str() == "true" || raw.to_lowercase().as_str() == "false" {
            4
        } else if raw.starts_with('[') && raw.ends_with(']') {
            5
        } else if raw.starts_with('{') && raw.ends_with('}') {
            6
        } else {
            2
        }
    }

    pub fn load(t: u16, raw: String) -> Option<Self> {
        let raw = raw.trim().to_string();
        match t {
            1 => {
                let id = Uuid::parse_str(raw.as_str());
                if id.is_err() {
                    return None;
                }
                Some(DataType::Id(id.unwrap()))
            }
            2 => Some(DataType::Text(raw.trim_matches('"').to_string())),
            3 => {
                let n = raw.parse::<i32>();
                if n.is_err() {
                    return None;
                }
                Some(DataType::Number(n.unwrap()))
            }
            4 => match raw.to_lowercase().as_str() {
                "true" => Some(DataType::Boolean(true)),
                "false" => Some(DataType::Boolean(false)),
                _ => None,
            },
            5 => {
                let mut new_vec = Vec::new();
                let raw = raw.strip_suffix(']').unwrap().strip_prefix('[').unwrap();
                let mut open_array = false;
                let mut open_string = false;
                let mut sub_raw = String::new();
                for chr in raw.chars() {
                    if chr == ',' && !open_array && !open_string {
                        let t = Self::infer_type(&sub_raw);
                        let r = Self::load(t, sub_raw.clone());
                        if r.is_some() {
                            new_vec.push(r.unwrap());
                            sub_raw = String::new();
                            continue;
                        }
                    }
                    if chr == '[' && !open_array {
                        open_array = true;
                    }
                    if chr == ']' && open_array {
                        open_array = false;
                    }
                    if chr == '"' {
                        open_string = !open_string
                    }
                    sub_raw.push(chr);
                }
                if !sub_raw.is_empty() {
                    let t = Self::infer_type(&sub_raw);
                    let r = Self::load(t, sub_raw.clone());
                    if r.is_some() {
                        new_vec.push(r.unwrap());
                    }
                }

                Some(DataType::Array(new_vec))
            }
            6 => {
                let mut d = Document::new();
                let raw = raw.strip_suffix('}').unwrap().strip_prefix('{').unwrap();
                let mut key = String::new();
                let mut key_done = false;
                let mut open_array = false;
                let mut open_string = false;
                let mut value = String::new();
                for chr in raw.chars() {
                    if chr == ':' && !key_done {
                        key_done = true;
                        continue;
                    }
                    if key_done {
                        if chr == ',' && !open_array && !open_string {
                            let t = Self::infer_type(&value);
                            let r = Self::load(t, value.clone());
                            if r.is_some() {
                                d.insert(key, r.unwrap());
                                key = String::new();
                                value = String::new();
                                key_done = false;
                                continue;
                            }
                        }

                        if chr == '[' && !open_array {
                            open_array = true;
                        }
                        if chr == ']' && open_array {
                            open_array = false;
                        }
                        if chr == '"' {
                            open_string = !open_string
                        }

                        value.push(chr);
                    } else {
                        key.push(chr);
                    }
                }
                Some(DataType::Document(d))
            }
            _ => None,
        }
    }
}

impl ToString for DataType {
    fn to_string(&self) -> String {
        match self {
            DataType::Id(id) => id.to_string(),
            DataType::Text(text) => format!("\"{}\"", text.to_string()),
            DataType::Number(number) => number.to_string(),
            DataType::Boolean(boolean) => boolean.to_string(),
            DataType::Array(array) => {
                let mut result = String::new();
                result.push('[');
                for value in array {
                    result.push_str(&value.to_string());
                    result.push_str(", ");
                }
                let mut result = result.strip_suffix(", ").unwrap().to_string();
                result.push(']');
                result
            }
            DataType::Document(document) => {
                let mut result = String::new();
                result.push('{');
                for (key, value) in document {
                    result.push_str(&key);
                    result.push_str(": ");
                    result.push_str(&value.to_string());
                    result.push_str(", ");
                }
                result.push('}');

                result
            }
        }
    }
}

impl From<Uuid> for DataType {
    fn from(value: Uuid) -> Self {
        DataType::Id(value)
    }
}

impl From<String> for DataType {
    fn from(value: String) -> Self {
        DataType::Text(value)
    }
}

impl From<&str> for DataType {
    fn from(value: &str) -> Self {
        DataType::Text(value.to_string())
    }
}

impl From<i32> for DataType {
    fn from(value: i32) -> Self {
        DataType::Number(value)
    }
}

impl From<bool> for DataType {
    fn from(value: bool) -> Self {
        DataType::Boolean(value)
    }
}

impl From<Vec<DataType>> for DataType {
    fn from(value: Vec<DataType>) -> Self {
        DataType::Array(value)
    }
}

impl From<Document> for DataType {
    fn from(value: Document) -> Self {
        DataType::Document(value)
    }
}

//impl clone
impl Clone for DataType {
    fn clone(&self) -> Self {
        match self {
            DataType::Id(id) => DataType::Id(*id),
            DataType::Text(text) => DataType::Text(text.clone()),
            DataType::Number(number) => DataType::Number(*number),
            DataType::Boolean(boolean) => DataType::Boolean(*boolean),
            DataType::Array(array) => DataType::Array(array.clone()),
            DataType::Document(document) => DataType::Document(document.clone()),
        }
    }
}
