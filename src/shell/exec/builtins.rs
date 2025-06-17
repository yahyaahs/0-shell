use std::{collections::HashMap, env, fs};
use super::list;

pub fn get_builtins() -> HashMap<String, fn(&Vec<String>)> {
    HashMap::from([
        ("exit".to_string(), exit as fn(&Vec<String>)),
        ("echo".to_string(), echo as fn(&Vec<String>)),
        ("ls".to_string(), list::ls as fn(&Vec<String>)),
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

fn exit(args: &Vec<String>) {
    if args.len() == 0 {
        std::process::exit(0)
    };
    match args[0].parse::<i32>() {
        Ok(nb) => std::process::exit(nb),
        Err(_) => std::process::exit(0),
    };
}

fn echo(args: &Vec<String>) {
    println!("{}", args.join(" "));
}
