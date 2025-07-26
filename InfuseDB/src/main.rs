mod arg_parser;
mod infusedb;
mod repl;
#[cfg(feature = "server")]
mod server;

use arg_parser::{ArgSearch, args_parser};
use repl::command::Command;
use repl::commander;
use repl::commander::Commander;
use repl::help_const;
#[cfg(feature = "server")]
use server::Server;

use infusedb::{DataType, InfuseDB, VERSION, utils};

use std::path::Path;

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
    let collection_name = args
        .get_key("-c")
        .unwrap_or(DEFAULT_COLLECTION_NAME.to_string());

    if !Path::new(&path).exists() {
        db.path = path;
    } else {
        db = InfuseDB::load(&path).unwrap();
    }
    println!("InfuseDB {}", VERSION);
    if db.get_collection(&collection_name).is_none() {
        let _ = db.create_collection(&collection_name);
    }

    if args.count_simple() == 0 {
        let mut commander = Commander::new(db);
        commander.repl_loop();
    } else {
        #[cfg(feature = "server")]
        if args.get_key("-s").is_some() {
            let mut server = Server::new("0.0.0.0", 1234).expect("vaia");
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
            Err(err) => format!("{:?}", err.to_string()),
        };
        println!("{}", output);
        let _ = db.dump();
    }
}
