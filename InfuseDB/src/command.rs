use infusedb::utils;

use crate::doc;
use crate::infusedb::{Collection, DataType};

pub trait Command {
    fn run(&mut self, command: &str) -> Result<DataType, &'static str>;
}

impl Command for Collection {
    fn run(&mut self, command: &str) -> Result<DataType, &'static str> {
        let command: Vec<String> = utils::smart_split(command.to_string());
        let action = command.get(0).ok_or("Empty command")?;
        let args: Vec<String> = command.iter().skip(1).cloned().collect();
        return match action.as_str() {
            "list" => Ok(DataType::Document(self.list())),
            "count" => Ok(DataType::Number(self.count() as f32)),
            "set" => {
                if args.len() < 2 {
                    return Err("No enought args");
                }
                let key = args.get(0).unwrap().as_str();
                let value = args.get(1).unwrap().to_string();
                let t = DataType::infer_type(&value);
                let d = DataType::load(t, value).ok_or("Unable to parse value")?;
                self.add(key, d);
                Ok(DataType::Boolean(true))
            }
            "get" => {
                if args.len() < 1 {
                    return Err("No enought args");
                }
                let key = args.get(0).unwrap().as_str();
                let value = self.get(key).ok_or("Key don't exists")?;
                if args.len() >= 1 {
                    let mut name = "key".to_string();
                    let mut value_pointer = value;
                    for arg_i in 1..args.len() {
                        let i = args.get(arg_i).unwrap().parse::<usize>();
                        if i.is_err() {
                            break;
                        }
                        let i = i.unwrap();
                        value_pointer = match &value_pointer {
                            DataType::Array(v) => {
                                let result = v.get(i);
                                if result.is_none() {
                                    return Ok(value_pointer.clone());
                                }
                                result.unwrap()
                            }
                            _ => break,
                        };
                        name.push_str(&format!("[{}]", i));
                    }
                    return Ok(value_pointer.clone());
                } else {
                    return Ok(value.clone());
                };
            }
            "del" => {
                if args.len() < 1 {
                    return Err("No enought args");
                }
                let key = args.get(0).unwrap().as_str();
                self.rm(key);
                Ok(DataType::Boolean(true))
            }
            "name" => Ok(doc!("name" => self.name.clone())),
            _ => Err("Unknown command"),
        };
    }
}
