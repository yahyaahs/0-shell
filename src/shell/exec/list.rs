use std::{ffi::OsString, fs, fs::DirEntry, os::unix::fs::PermissionsExt};

use crate::shell::Shell;

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
pub fn ls(_shell: &Shell, args: &Vec<String>) {
    let paths = fs::read_dir(".").unwrap();
    let mut output = vec![];
    let show = args.contains(&"-a".to_string());
    let classify = args.contains(&"-F".to_string());
    let blue = "\x1b[34m";
    let green = "\x1b[32m";
    let reset = "\x1b[0m";
    if show {
        output.push(".".to_string());
        output.push("..".to_string());
    }
    for data in paths {
        match check_type(data.unwrap()) {
            Types::Dir(name) => {
                let name_str = name.to_string_lossy();
                let display = if classify {
                    format!("{}/", name_str)
                } else {
                    name_str.to_string()
                };
                let colored = format!("{}{}{}", blue, display, reset);
                if show {
                    output.push(colored);
                } else if !name_str.starts_with('.') {
                    output.push(colored);
                }
            }
            Types::Executable(name) => {
                let name_str = name.to_string_lossy();
                if show {
                    output.push(format!("{}{}{}", green, name_str, reset));
                } else if !name_str.starts_with('.') {
                    output.push(format!("{}{}{}", green, name_str, reset));
                }
            }
            Types::File(name) | Types::Symlink(name) => {
                let name_str = name.to_string_lossy();
                if show {
                    output.push(name_str.to_string());
                } else if !name_str.starts_with('.') {
                    output.push(name_str.to_string());
                }
            }
            _ => {}
        }
    }
    for item in output {
        println!("{}", item);
    }
}
