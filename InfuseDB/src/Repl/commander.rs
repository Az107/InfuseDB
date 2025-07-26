use crate::InfuseDB;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use super::command::Command;
use super::help_const::HELP_STR_MAIN;
use crate::format_data_type;
use crate::infusedb::utils;

enum CommandErr {
    NotFound,
}

pub struct Commander {
    // commands: HashMap<String, Cmd>,
    db: InfuseDB,
    selected: String,
}

impl Commander {
    pub fn new(db: InfuseDB) -> Commander {
        Commander {
            db,
            selected: String::new(),
            // commands: HashMap::new(),
        }
    }

    pub fn repl_loop(&mut self) {
        let rl = DefaultEditor::new();
        if rl.is_err() {
            println!("Error opening REPL");
            return;
        }
        let mut rl = rl.unwrap();
        loop {
            let prompt = format!("{}> ", self.selected);
            let readline = rl.readline(&prompt);
            let readline = match readline {
                Ok(line) => {
                    let _ = rl.add_history_entry(line.as_str());
                    line
                }
                Err(ReadlineError::Interrupted) => {
                    println!("Bye");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            };

            let command: Vec<String> = utils::smart_split(readline.clone());
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

            let output = match action.as_str() {
                "help" => Some(HELP_STR_MAIN.to_string()),
                "exit" => {
                    let _ = self.db.dump();
                    break;
                }
                "list" => {
                    let mut result = "".to_string();
                    for col_name in self.db.get_collection_list() {
                        result.push_str(format!("-> {}", col_name).as_str());
                    }
                    Some(result)
                }
                "select" => {
                    if let Some(col_name) = args.get(0) {
                        if self.db.get_collection_list().contains(&col_name) {
                            self.selected = col_name.to_string();
                            Some(String::new())
                        } else {
                            Some(format!("Collection {} dont exists", col_name))
                        }
                    } else {
                        Some("usage: select <collection name>".to_string())
                    }
                }
                "new" => {
                    if let Some(col_name) = args.get(0) {
                        if self.db.get_collection_list().contains(&col_name) {
                            Some(format!("Collection {} already exists", col_name))
                        } else {
                            let _ = self.db.create_collection(&col_name);
                            Some(format!("Collection {} created", col_name))
                        }
                    } else {
                        Some("usage: new <collection name>".to_string())
                    }
                }
                "del" => {
                    if !self.selected.is_empty() {
                        None
                    } else {
                        if let Some(col_name) = args.get(0) {
                            if self.db.get_collection_list().contains(col_name) {
                                self.db.remove_collection(col_name.to_string());
                                Some("".to_string())
                            } else {
                                Some(format!("Collection {} dont exists", col_name))
                            }
                        } else {
                            Some("usage: select <collection name>".to_string())
                        }
                    }
                }
                _ => None,
            };

            let output = if output.is_none() && !self.selected.is_empty() {
                let collection = self.db.get_collection(self.selected.as_str()).unwrap();

                let r = collection.run(&readline);
                match r {
                    Ok(result) => format!("{}", format_data_type(result, 0)),
                    Err(err) => format!("error: {:?}", err.to_string()),
                }
            } else {
                output.unwrap_or(String::new()).to_string()
            };
            if !output.is_empty() {
                println!("{}", output);
            }
        }
    }
}
