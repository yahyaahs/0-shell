use std::{fs::{self, DirEntry}, os::unix::fs::{MetadataExt, PermissionsExt}};

use super::helper::{Types, check_type};
use crate::shell::Shell;
pub fn list_arg(args: &mut DirEntry) ->String {
    println!("{:#?}", args.metadata());
    let mode = args.metadata().unwrap().permissions().mode();
    let file_type = match mode & 0o170000 {
        0o040000 => 'd', // directory
        0o100000 => '-', // regular file
        0o120000 => 'l', // symlink
        _ => '?',        // other
    };

    let mut perms = String::new();
    perms.push(file_type);

    // User permissions
    perms.push(if mode & 0o400 != 0 { 'r' } else { '-' },);
    perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    // Group permissions
    perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    // Others permissions
    perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    perms
}

pub fn ls(_shell: &mut Shell, args: &Vec<String>) {
    let paths = fs::read_dir(".").unwrap();
    let mut output = vec![];
    let show = args.contains(&"a".to_string());
    let classify = args.contains(&"F".to_string());
    let mut perms = String::new();
    let blue = "\x1b[34m";
    let green = "\x1b[32m";
    let reset = "\x1b[0m";
    if show {
        output.push(".".to_string());
        output.push("..".to_string());
    }
    for data in paths {
        let mut elems = data.unwrap();
    if args.contains(&"l".to_string()){
        perms = list_arg(&mut elems);

    }
        match check_type(&elems) {
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
