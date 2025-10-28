mod arg_parser;
mod command;
mod help_const;
mod infusedb;
#[cfg(feature = "server")]
mod server;

#[cfg(feature = "server")]
use server::Server;

use arg_parser::{ArgSearch, args_parser};
use command::Command;
use infusedb::{DataType, InfuseDB, VERSION, utils};

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{env, io};

const DEFAULT_PATH: &str = "~/.infusedb/default.mdb";
const DEFAULT_COLLECTION_NAME: &str = "default";

fn format_data_type(data: DataType, sub: u32) -> String {
    match data {
        DataType::Document(doc) => {
            let mut r = String::new();
            if sub > 0 {
                r.push('\n');
            }
            for (key, val) in doc {
                for _ in 0..sub {
                    r.push(' ');
                }
                r.push_str(&format!("{}: {}\n", &key, &format_data_type(val, sub + 1)));
            }

            r
        }
        // DataType::Array(list) => format!("[{}]", format_data_type(list[0].clone())),
        _ => data.to_string(),
    }
}

fn main() {
    let mut db = InfuseDB::new();
    let args = args_parser();
    let path = args.get_key("-p").unwrap_or(DEFAULT_PATH.to_string());
    let home = env::home_dir().unwrap();
    let home = home.to_str().unwrap();
    let path = path.replace("~", home);
    let collection_name = args
        .get_key("-c")
        .unwrap_or(DEFAULT_COLLECTION_NAME.to_string());

    if !Path::new(&path).exists() {
        db.path = path.to_string();
    } else {
        db = InfuseDB::load(&path).unwrap();
    }
    println!("InfuseDB {}", VERSION);
    if db.get_collection(&collection_name).is_none() {
        let _ = db.create_collection(&collection_name);
    }
    let mut selected = String::new();
    if args.count_simple() == 0 {
        loop {
            print!("{}> ", selected);
            let _ = io::stdout().flush();
            let mut buffer = String::new();
            let _ = io::stdin().read_line(&mut buffer);
            let command: Vec<String> = utils::smart_split(buffer.clone());
            let action = command.get(0);
            if action.is_none() {
                continue;
            }
            let action = action.unwrap();
            let args = if command.len() > 0 {
                command.clone()[1..].to_vec()
            } else {
                Vec::new()
            };

            if action == "exit" {
                break;
            } else if action == "select" {
                if args.len() >= 1 && db.get_collection_list().contains(&args[0]) {
                    selected = args[0].clone()
                } else {
                    println!("Collection don't exists");
                }
                continue;
            } else if action == "deselect" && !selected.is_empty() {
                selected = String::new();
                continue;
            } else if selected.is_empty() && action == "list" {
                for c in db.get_collection_list() {
                    println!("-> {}", c);
                }
                continue;
            } else if action == "del_col" {
                if selected.is_empty() {
                    if args.len() != 0 {
                        selected = args[0].clone();
                    } else {
                        println!("No collection selected");
                        continue;
                    }
                }
                db.remove_collection(selected);
                selected = String::new();
            } else if action == "new" {
                if args.len() != 0 {
                    let _ = db.create_collection(&args[0]);
                } else {
                    println!("No collection name provided");
                }
                continue;
            } else if action == "commit" {
                let r = db.dump();
                if r.is_err() {
                    println!("Error saving changes: {}", r.err().unwrap());
                } else {
                    println!("Changed saved");
                }
                continue;
            } else if action == "help" {
                if selected.is_empty() {
                    println!("{}", help_const::HELP_STR_MAIN);
                } else {
                    println!("{}", help_const::HELP_STR_COL);
                }
                continue;
            }

            if selected == "" {
                println!("No collection selected");
                continue;
            }
            let collection = db.get_collection(selected.as_str()).unwrap();
            let r = collection.run(&buffer);
            let output = match r {
                Ok(result) => format!("{}", format_data_type(result, 0)),
                Err(err) => format!("{:?}", err.to_string()),
            };
            println!("{}", output);
        }
    } else {
        #[cfg(feature = "server")]
        if args.get_key("-s").is_some() {
            let mut server = Server::new("0.0.0.0", 1234, db).expect("vaia");
            println!("Starting server on 1234");
            let _ = server.listen();

            return;
        }

        let commands = args.get_single_joined();
        let command = commands.last();
        if command.is_none() {
            return;
        }
        let command = command.unwrap();

        if command == "help" {
            println!("{}", help_const::HELP_STR_COL);
            return;
        }
        let collection = db.get_collection(&collection_name).unwrap();
        let r = collection.run(&command);
        let output = match r {
            Ok(result) => format!("{}", format_data_type(result, 0)),
            Err(err) => format!("{}", err.to_string()),
        };
        println!("{}", output);
    }

    let _ = db.dump();
}
