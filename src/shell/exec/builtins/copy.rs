use super::*;

use std::{
    fs::{self, metadata},
    io::*,
    os::unix::fs::PermissionsExt,
};

pub fn cp(_shell: &mut Shell, command: &Cmd) {
    if command.args.len() < 2 {
        println!("usage: cp source_file target_file\n       cp source_file ... target_directory");
        return;
    }
    println!("{:?}", command);
    let sources = command.args[0..command.args.len() - 1].to_vec();
    let target = &command.args[command.args.len() - 1];
    if sources.len() == 1 {
        one_source(&sources[0], &command.exec, target);
    } else {
        let data_of_target = match metadata(target) {
            Ok(data) => data,
            Err(error) => {
                println!("{:?}", error);
                return;
            }
        };
        if !data_of_target.is_dir() {
            println!("{}: {} {}", command.exec, target, "is not a directory");
            return;
        }
        for source in &sources {
            let data_of_source = match metadata(source) {
                Ok(data) => data,
                Err(error) => {
                    println!("{:?}", error);
                    return;
                }
            };
            if data_of_source.is_dir() {
                println!(
                    "{}: {} {}",
                    command.exec, source, "is a directory (not copied)."
                );
                continue;
            }
            let content: String = match fs::read_to_string(source) {
                Ok(data) => data,
                Err(error) => {
                    eprintln!("Error reading file: {}", error);
                    return;
                }
            };
            match create_file(target, &content, source, &command.exec) {
                Some(path) => {
                    copy_perms(source, &path);
                }
                None => return
            }
        }
    }
}

pub fn one_source(source: &String, command: &String, target: &String) {
    if source == target {
        println!(
            "{}: {} and {} {}",
            command, source, target, "are identical (not copied)."
        );
        return;
    }
    let data_of_source = match metadata(&source) {
        Ok(data) => data,
        Err(error) => {
            println!("{:?}", error);
            return;
        }
    };
    if data_of_source.is_dir() {
        println!("{}: {} {}", command, source, "is a directory (not copied).");
        return;
    }
    let content: String = match fs::read_to_string(source) {
        Ok(data) => data,
        Err(error) => match error.kind() {
            ErrorKind::PermissionDenied => {
                println!("{}: {}: {}", command, source, "Permission denied");
                return;
            }
            _ => {
                eprintln!("Error reading file: {}", error);
                return;
            }
        },
    };
    match create_file(target, &content, source, command) {
        Some(path) => {
            copy_perms(source, &path);

        }
        None => return
    }
}

pub fn create_file(path: &String, content: &String, source: &String, command: &String) -> Option<String> {
    match fs::write(path, content.trim()) {
        Ok(_) => Some(path.clone()),
        Err(error) => match error.kind() {
            ErrorKind::IsADirectory => {
                let new_path = &format!("{}/{}", path, source);
                create_file(new_path, content, source, command);
                return Some(new_path.clone());
            }
            ErrorKind::PermissionDenied => {
                println!("{}: {}: {}", command, path, "Permission denied");
                None
            }
            ErrorKind::NotADirectory => {
                println!(
                    "{}: {}: {}",
                    command,
                    path.trim_end_matches("/"),
                    "is not a directory"
                );
                return None;
            }
            ErrorKind::NotFound => {
                println!("{}: {}: {}", command, path, "Not Fount");
                return None;
            }
            _ => {
                return None;
            }
        },
    };
    Some(path.clone())
}

pub fn copy_perms(source: &String, target: &String) {
    let data_of_source = match metadata(&source) {
        Ok(data) => data,
        Err(error) => {
            println!("{:?}", error);
            return;
        }
    };
    let src_perms = data_of_source.permissions();
    let data_of_target = match metadata(&target) {
        Ok(data) => data,
        Err(error) => {
            println!("{:?}", error);
            return;
        }
    };
    let mut target_perms = data_of_target.permissions();
    target_perms.set_mode(src_perms.mode());
    let _ = fs::set_permissions(target, target_perms);
}
