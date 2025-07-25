use std::collections::HashMap;

use infusedb::utils;

use crate::doc;
use crate::infusedb::{Collection, DataType, FindOp};

pub trait Command {
    fn run(&mut self, command: &str) -> Result<DataType, CommandError>;
}

pub enum CommandError {
    EmptyCommand,
    NoEnoughArgs,
    UnknownCommand,
    ErrorParsing,
    KeyNotFound(String, String),
    Custom(&'static str),
}

impl CommandError {
    pub fn to_string(&self) -> String {
        match self {
            CommandError::EmptyCommand => "Command is empty".to_string(),
            CommandError::NoEnoughArgs => "No enough args".to_string(),
            CommandError::UnknownCommand => "Command does not exists".to_string(),
            CommandError::ErrorParsing => "Error parsing".to_string(),
            CommandError::KeyNotFound(key, parent) => {
                format!("Key {} does not exist in {}", key, parent)
            }
            CommandError::Custom(custom) => format!("Unknown error: {}", custom),
        }
    }
}

impl Command for Collection {
    fn run(&mut self, command: &str) -> Result<DataType, CommandError> {
        let command: Vec<String> = utils::smart_split(command.to_string());
        let action = command.get(0).ok_or(CommandError::EmptyCommand)?;
        let args: Vec<String> = command.iter().skip(1).cloned().collect();
        return match action.as_str() {
            "list" => Ok(DataType::Document(self.list())),
            "count" => Ok(DataType::Number(self.count() as f32)),
            "set" => {
                if args.len() < 2 {
                    return Err(CommandError::NoEnoughArgs);
                }
                let key = args.get(0).unwrap().as_str();
                let keys: Vec<&str> = key.split(".").collect();
                let value = args.get(1).unwrap().to_string();
                let t = DataType::infer_type(&value);
                let d = DataType::load(t, value).ok_or(CommandError::ErrorParsing)?;
                let mut parent = &mut self.data;
                let inter_keys = keys[0..keys.len() - 1].to_vec();
                for i in 0..inter_keys.len() {
                    let k = inter_keys[i];
                    let p = parent.get(k);
                    if p.is_none() {
                        let dt = if i + 1 < inter_keys.len() {
                            let k1 = inter_keys[i + 1];
                            if k1.parse::<usize>().is_ok() {
                                DataType::Array(Vec::new())
                            } else {
                                DataType::Document(HashMap::new())
                            }
                        } else {
                            DataType::Document(HashMap::new())
                        };
                        let _ = parent.set(k, dt);
                    }
                    parent = parent.get_mut(k).unwrap();
                }

                parent
                    .set(
                        keys.last()
                            .ok_or(CommandError::KeyNotFound("?".to_string(), "??".to_string()))?,
                        d,
                    )
                    .map_err(|_| CommandError::Custom("()"))
            }
            "get" => {
                // get key.path [where <subkey> <is|not is|gr|ls> <value>]

                if args.len() < 1 {
                    return Err(CommandError::NoEnoughArgs);
                }
                let proto_key = args.get(0).unwrap().as_str();
                let is_search = args.len() == 5 && args[1] == "where"; //TODO: improve the comparison selector

                let keys: Vec<&str> = proto_key.split('.').collect();
                let value = self.get(keys[0]).ok_or(CommandError::KeyNotFound(
                    keys[0].to_string(),
                    "Collection".to_string(),
                ))?;

                let get_result = if proto_key.contains('.') {
                    let mut name = "key".to_string();
                    let mut value_pointer = value;

                    for arg_i in 1..keys.len() {
                        let i = keys.get(arg_i).unwrap().to_string();
                        value_pointer = match &value_pointer {
                            DataType::Array(v) => {
                                let i = i.parse::<usize>();
                                if i.is_err() {
                                    break;
                                }
                                let i = i.unwrap();
                                let result = v.get(i);
                                if result.is_none() {
                                    return Err(CommandError::KeyNotFound(
                                        keys[arg_i].to_string(),
                                        keys[arg_i - 1].to_string(),
                                    ));
                                }
                                result.unwrap()
                            }
                            DataType::Document(d) => {
                                let result = d.get(&i);
                                if result.is_none() {
                                    return Err(CommandError::KeyNotFound(
                                        keys[arg_i].to_string(),
                                        keys[arg_i - 1].to_string(),
                                    ));
                                }
                                result.unwrap()
                            }
                            _ => break,
                        };
                        name.push_str(&format!("[{}]", i));
                    }

                    value_pointer.clone()
                } else {
                    value.clone()
                };
                if is_search {
                    let sub_key = args[2].clone();
                    let t = DataType::infer_type(&args[4]);
                    let v = DataType::load(t, args[4].clone());
                    if v.is_none() {
                        return Err(CommandError::ErrorParsing);
                    }
                    let value = v.unwrap();
                    let op = match args[3].as_str() {
                        "==" | "is" => FindOp::Eq,
                        "isnot" | "notis" | "!=" => FindOp::NotEq,
                        ">" | "gt" => FindOp::Gt,
                        "<" | "lt" => FindOp::Lt,
                        _ => return Err(CommandError::ErrorParsing),
                    };
                    match get_result.find(&sub_key, op, value) {
                        Some(d) => Ok(d.clone()),
                        None => Err(CommandError::KeyNotFound(sub_key, "Search".to_string())),
                    }
                } else {
                    Ok(get_result)
                }
            }
            "del" => {
                if args.len() < 1 {
                    return Err(CommandError::NoEnoughArgs);
                }
                let key = args.get(0).unwrap().as_str();
                self.rm(key);
                Ok(DataType::Boolean(true))
            }
            "name" => Ok(doc!("name" => self.name.clone())),
            _ => Err(CommandError::UnknownCommand),
        };
    }
}
