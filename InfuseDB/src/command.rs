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
        let action = command.first().ok_or(CommandError::EmptyCommand)?;
        let args: Vec<String> = command.iter().skip(1).cloned().collect();
        return match action.as_str() {
            "list" => Ok(DataType::Document(self.list())),
            "count" => Ok(DataType::Number(self.count() as f32)),
            "set" => {
                if args.len() < 2 {
                    return Err(CommandError::NoEnoughArgs);
                }
                let key = args.get(0).unwrap().as_str();
                let value = args.get(1).unwrap().to_string();
                let t = DataType::infer_type(&value);
                let d = DataType::load(t, value).ok_or(CommandError::ErrorParsing)?;
                match self.set(key, d) {
                    Ok(_) => Ok(DataType::Text("Ok".to_string())),
                    Err(err) => {
                        println!("Error adding: {:?}", err);
                        Err(CommandError::Custom("Error adding"))
                    }
                }
            }
            "get" => {
                // get key.path [where <subkey> <is|not is|gr|ls> <value>]

                if args.len() < 1 {
                    return Err(CommandError::NoEnoughArgs);
                }
                let key = args.get(0).unwrap().as_str();
                self.get(key).ok_or(CommandError::KeyNotFound(
                    key.to_owned(),
                    "collection".to_owned(),
                ))
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
