use crate::shell::Shell;

use super::list;
use std::{collections::HashMap, env};

pub fn get_builtins() -> HashMap<String, fn(&Shell, &Vec<String>)> {
    HashMap::from([
        ("exit".to_string(), exit as fn(&Shell, &Vec<String>)),
        ("echo".to_string(), echo as fn(&Shell, &Vec<String>)),
        ("ls".to_string(), list::ls as fn(&Shell, &Vec<String>)),
    ])
}

pub fn find_non_builtins(cmd: &str) -> Option<String> {
    let path = match env::var("PATH") {
        Ok(path) => path,
        Err(_) => return None,
    };

    for dir in path.split(":") {
        let entries = std::fs::read_dir(&dir);
        if let Ok(entries) = entries {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() {
                    if let Some(path_str) = path.to_str() {
                        if path_str.to_string().ends_with(&("/".to_owned() + cmd)) {
                            return Some(path_str.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

fn exit(_shell: &Shell, args: &Vec<String>) {
    if args.len() == 0 {
        std::process::exit(0)
    };
    match args[0].parse::<i32>() {
        Ok(nb) => std::process::exit(nb),
        Err(_) => std::process::exit(0),
    };
}

fn echo(_shell: &Shell, args: &Vec<String>) {
    println!("{}", args.join(" "));
}
