use super::*;

use std::{
    fs::{self, metadata},
    io::*,
    os::unix::fs::PermissionsExt,
};

pub fn cp(shell: &mut Shell, command: &Cmd) {
    if command.args.len() < 2 {
        eprintln!("usage: cp source_file target_file\n       cp source_file ... target_directory");
        return;
    }
    let sources = command.args[0..command.args.len() - 1].to_vec();
    let target = &command.args[command.args.len() - 1];
    // let curent_dir = &shell.cwd.to_string_lossy().to_string();
    // if target == "." || target == "./"{
    //     println!("hanni");
    //     target = curent_dir;
    // }
    // println!("{}",target);
    if sources.len() == 1 {
        // let source : Vec<String> = sources[0].split("/").map(|f| f.to_string()).collect();
        one_source(&sources[0], &command.exec, target);
    } else {
        let data_of_target = match metadata(target) {
            Ok(data) => data,
            Err(error) => {
                eprintln!("{:?}", error);
                return;
            }
        };
        if !data_of_target.is_dir() {
            eprintln!("{}: {} {}", command.exec, target, "is not a directory");
            return;
        }
        for source in &sources {
            let data_of_source = match metadata(source) {
                Ok(data) => data,
                Err(error) => {
                    eprintln!("{:?}", error);
                    return;
                }
            };
            if data_of_source.is_dir() {
                eprintln!(
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
        eprintln!(
            "{}: {} and {} {}",
            command, source, target, "are identical (not copied)."
        );
        return;
    }
    let data_of_source = match metadata(&source) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("{:?}", error);
            return;
        }
    };
    if data_of_source.is_dir() {
        eprintln!("{}: {} {}", command, source, "is a directory (not copied).");
        return;
    }
    let content: String = match fs::read_to_string(source) {
        Ok(data) => data,
        Err(error) => match error.kind() {
            ErrorKind::PermissionDenied => {
                eprintln!("{}: {}: {}", command, source, "Permission denied");
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
    let s : Vec<String> = source.split("/").map(|f| f.to_string()).collect();
    let s1 = &s[s.len() -1];
    match fs::write(path, content.trim()) {
        Ok(_) => Some(path.clone()),
        Err(error) => match error.kind() {
            ErrorKind::IsADirectory => {
                if path.ends_with(s1) {
                    eprintln!("cp: cannot overwrite directory {} with non-directory {}", format!("{}/{}",path,s1), source);
                    return None
                }
                let new_path = &format!("{}/{}", path, &s1);

                match create_file(new_path, content, source, command) {
                    Some(_) => {}
                    None => return None
                }
                return Some(new_path.clone());
            }
            ErrorKind::PermissionDenied => {
                eprintln!("{}: {}: {}", command, path, "Permission denied");
                None
            }
            ErrorKind::NotADirectory => {
                eprintln!(
                    "{}: {}: {}",
                    command,
                    path.trim_end_matches("/"),
                    "is not a directory"
                );
                return None;
            }
            ErrorKind::NotFound => {
                eprintln!("{}: {}: {}", command, path, "Not Fount");
                return None;
            }
            _ => {
                eprintln!("oui {}",error);
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
            eprintln!("{:?}", error);
            return;
        }
    };
    let src_perms = data_of_source.permissions();
    let data_of_target = match metadata(&target) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("{:?}", error);
            return;
        }
    };
    let mut target_perms = data_of_target.permissions();
    target_perms.set_mode(src_perms.mode());
    let _ = fs::set_permissions(target, target_perms);
}
