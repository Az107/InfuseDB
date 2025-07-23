mod command;
mod help_const;
mod infusedb;
#[cfg(feature = "server")]
mod server;

#[cfg(feature = "server")]
use server::Server;

use command::Command;
use infusedb::{utils, DataType, InfuseDB, VERSION};

use std::io::Write;
use std::path::Path;
use std::{env, io};

const DEFAULT_PATH: &str = "default.mdb";
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
    if !Path::new(DEFAULT_PATH).exists() {
        db.path = DEFAULT_PATH.to_string();
    } else {
        db = InfuseDB::load(DEFAULT_PATH).unwrap();
    }
    println!("InfuseDB {}", VERSION);
    if db.get_collection(DEFAULT_COLLECTION_NAME).is_none() {
        let _ = db.create_collection(DEFAULT_COLLECTION_NAME);
    }
    let mut selected = String::new();
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
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
        if args[1] == "-s" {
            let mut server = Server::new("0.0.0.0", 1234).expect("vaia");
            println!("Starting server on 1234");
            let _ = server.listen();

            return;
        }
        let command = args.clone()[1..].to_vec().join(" ");
        let collection = db.get_collection(DEFAULT_COLLECTION_NAME).unwrap();
        if command == "help" {
            println!("{}", help_const::HELP_STR_COL);
            return;
        }
        let r = collection.run(&command);
        let output = match r {
            Ok(result) => format!("{}", format_data_type(result, 0)),
            Err(err) => format!("{:?}", err.to_string()),
        };
        println!("{}", output);
    }

    let _ = db.dump();
}
