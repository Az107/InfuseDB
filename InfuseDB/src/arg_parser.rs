use std::env::{self};

pub enum Arg {
    Simple(String),
    Couple(String, String),
}

type BetterArg = Vec<Arg>;

pub trait ArgSearch {
    fn get_key(&self, key: &str) -> Option<String>;
    fn get_index(&self, index: usize) -> Option<(String, String)>;
    // fn get_range(&self, start: usize, end: isize) -> Option<String>;
    fn get_single_joined(&self) -> Vec<String>;
    fn count_simple(&self) -> usize;
    fn count_couple(&self) -> usize;
}

impl ArgSearch for BetterArg {
    fn get_key(&self, key: &str) -> Option<String> {
        for arg in self {
            let found = match arg {
                Arg::Simple(a) => {
                    if a == key {
                        Some(a.clone())
                    } else {
                        None
                    }
                }
                Arg::Couple(k, v) => {
                    if k == key {
                        Some(v.clone())
                    } else {
                        None
                    }
                }
            };
            if found.is_some() {
                return found;
            }
        }
        None
    }

    fn get_index(&self, index: usize) -> Option<(String, String)> {
        let r = self.get(index);
        if r.is_none() {
            return None;
        }

        let r = r.unwrap();
        let touple = match r {
            Arg::Simple(a) => (a.clone(), String::new()),
            Arg::Couple(k, v) => (k.clone(), v.clone()),
        };

        Some(touple)
    }

    fn count_simple(&self) -> usize {
        let mut count = 0;
        for arg in self {
            match arg {
                Arg::Simple(_) => count += 1,
                Arg::Couple(_, _) => {}
            }
        }
        return count;
    }

    fn count_couple(&self) -> usize {
        let mut count = 0;
        for arg in self {
            match arg {
                Arg::Simple(_) => {}
                Arg::Couple(_, _) => count += 1,
            }
        }
        return count;
    }

    fn get_single_joined(&self) -> Vec<String> {
        let mut result = Vec::new();
        let mut arg = String::new();

        for item in self {
            match item {
                Arg::Simple(a) => {
                    arg.push_str(&a);
                    arg.push_str(" ");
                }
                Arg::Couple(_, _) => {
                    if !arg.is_empty() {
                        result.push(arg.clone());
                        arg.clear();
                    }
                }
            }
        }
        if !arg.is_empty() {
            result.push(arg.clone());
        }

        result
    }
}

pub fn args_parser() -> BetterArg {
    let mut parsed = Vec::new();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg.starts_with('-') {
            let key = arg;
            let value = args.next();
            if value.is_some() {
                let value = value.unwrap();
                if value.starts_with('-') {
                    parsed.push(Arg::Simple(key));
                    parsed.push(Arg::Simple(value));
                } else {
                    parsed.push(Arg::Couple(key, value));
                }
            } else {
                parsed.push(Arg::Simple(key));
            }
        } else {
            parsed.push(Arg::Simple(arg));
        }
    }

    parsed
}
