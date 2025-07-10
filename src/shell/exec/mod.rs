pub mod builtins;

use crate::shell::Shell;
use crate::shell::exec::builtins::{mkdir, remove};
use crate::shell::parse::Cmd;

use std::{collections::HashMap, env};
use std::{ffi::OsString, fs::DirEntry, os::unix::fs::PermissionsExt};

pub use builtins::{
    base::{echo, exit, pwd},
    cat, cd, list,
};

use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, spawn};

#[derive(Debug)]
pub enum Types {
    File(OsString),
    Dir(OsString),
    Executable(OsString),
    Symlink(OsString),
    NotSupported,
    Error,
}

pub fn execute_command(shell: Arc<Mutex<Shell>>, command: Cmd) -> JoinHandle<()> {
    let shell_clone = Arc::clone(&shell);
    let command_clone = command.clone();

    spawn(move || {
        let mut shell_locked = match shell_clone.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };

        match shell_locked.builtins.get(&command_clone.exec) {
            Some(func) => func(&mut shell_locked, &command_clone),
            None => {
                println!("Command not found: {}", command_clone.exec);

                let bin_cmd = find_non_builtins(&command.exec);
                if let Some(bin) = bin_cmd {
                    println!("but we found this: {}", bin);
                }
            }
        }
    })
}

pub fn check_type(name: &DirEntry) -> Types {
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

pub fn get_builtins() -> HashMap<String, fn(&mut Shell, &Cmd)> {
    HashMap::from([
        ("exit".to_string(), exit as fn(&mut Shell, &Cmd)),
        ("echo".to_string(), echo as fn(&mut Shell, &Cmd)),
        ("pwd".to_string(), pwd as fn(&mut Shell, &Cmd)),
        ("ls".to_string(), list::ls as fn(&mut Shell, &Cmd)), // chang ls signature
        ("cd".to_string(), cd::cd as fn(&mut Shell, &Cmd)),
        ("cat".to_string(), cat::cat as fn(&mut Shell, &Cmd)),
        ("mkdir".to_string(), mkdir::mkdir as fn(&mut Shell, &Cmd)),
        ("rm".to_string(), remove::rm as fn(&mut Shell, &Cmd)),
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
