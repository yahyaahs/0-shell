use crate::shell::Shell;

use std::{collections::HashMap, env};
use std::{ffi::OsString, fs::DirEntry, os::unix::fs::PermissionsExt};

pub use super::builtins::{base, cd, list};

#[derive(Debug)]
pub enum Types {
    File(OsString),
    Dir(OsString),
    Executable(OsString),
    Symlink(OsString),
    NotSupported,
    Error,
}

pub fn check_type(name: DirEntry) -> Types {
    match name.metadata() {
        Ok(meta) => {
            if meta.is_dir() {
                return Types::Dir(name.file_name());
            } else if meta.permissions().mode() & 0o111 != 0 {
                return Types::Executable(name.file_name());
            } else if meta.is_file() {
                return Types::File(name.file_name());
            } else if meta.is_symlink() {
                return Types::Symlink(name.file_name());
            } else {
                return Types::NotSupported;
            }
        }
        _ => Types::Error,
    }
}

pub fn get_builtins() -> HashMap<String, fn(&mut Shell, &Vec<String>)> {
    HashMap::from([
        (
            "exit".to_string(),
            base::exit as fn(&mut Shell, &Vec<String>),
        ),
        (
            "echo".to_string(),
            base::echo as fn(&mut Shell, &Vec<String>),
        ),
        ("ls".to_string(), list::ls as fn(&mut Shell, &Vec<String>)),
        ("cd".to_string(), cd::cd as fn(&mut Shell, &Vec<String>)),
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
