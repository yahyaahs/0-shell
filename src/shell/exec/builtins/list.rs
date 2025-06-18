use std::fs;

use super::helper::{Types, check_type};
use crate::shell::Shell;

pub fn ls(_shell: &mut Shell, args: &Vec<String>) {
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
