use super::*;

use crate::shell::exec::{
    builtins::{
        copy::{copy_perms, create_file},
        mkdir::create_folder,
    },
    remove::demand_confirmation,
};

use std::{
    env,
    fs::{exists, metadata, read, read_dir, remove_dir, remove_dir_all, remove_file},
    io::*
};

pub fn mv(_: &mut Shell, command: &Cmd) {
    if command.args.len() < 2 {
        eprintln!("usage: mv source target\n\tmv source ... directory");
        return
    }
    let sources = &command.args[..command.args.len() - 1].to_vec();
    let mut target = command.args[command.args.len() - 1].clone();
    if target.starts_with("~") {
        let home = env::var("HOME");
        match home {
            Ok(p) => target = target.replace("~", &p),
            Err(_) => {
                write_("cd: cannot find HOME directory set\n");
                return;
            }
        }
    }
    
    if sources.len() > 1 {
        let data_of_target = match metadata(&target) {
            Ok(data) => data,
            Err(error) => match error.kind() {
                ErrorKind::PermissionDenied => {
                    eprintln!("{}: {}: {}", command.exec, target, "Permission denied");
                    return;
                }
                ErrorKind::NotADirectory => {
                    eprintln!(
                        "{}: {}: {}",
                        command.exec,
                        target.trim_end_matches("/"),
                        "is not a directory"
                    );
                    return;
                }
                ErrorKind::NotFound => {
                    eprintln!("{}: {}: {}", command.exec, target, "Not Found");
                    return;
                }
                _ => {
                    eprintln!("{}", error);
                    return;
                }
            },
        };
        if !data_of_target.is_dir() {
            eprintln!("{}: {} {}", command.exec, target, "is not a directory");
            return;
        }
    }
    for mut source in sources.clone() {
        if source.starts_with("~") {
            let home = env::var("HOME");
            match home {
                Ok(p) => source = source.replace("~", &p),
                Err(_) => {
                    write_("cd: cannot find HOME directory set\n");
                    return;
                }
            }
        }

        let is_exist = match exists(&source) {
            Ok(check) => check,
            Err(err) => {
                eprintln!("is_exist mv error {:?}", err);
                return;
            }
        };
        if !is_exist {
            eprintln!(
                "{}: rename {} to {}: No such file or directory",
                command.exec, source, target
            );
            continue;
        } else {
            let meta_data_source = match metadata(&source) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("metadata source mv error {:?}", err);
                    return;
                }
            };

            if meta_data_source.is_dir() {
                match move_dir(&source, &target, &command.exec) {
                    Some(_) => {}
                    None => return,
                }
            } else {
                match move_file(&source, &target, &command.exec) {
                    Some(_) => {}
                    None => return,
                }
            }
        }
    }
}

pub fn move_dir(s: &String, target: &String, comand: &String) -> Option<bool> {
    let target_meta_data = match metadata(target) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("error metadata in move dir{:?}", error);
            return None;
        }
    };
    if !target_meta_data.is_dir() {
        eprintln!("{}: rename {} to {}: Not a directory", comand, s, target);
        return None;
    }
    let holder: Vec<String> = s.split("/").map(|f| f.to_string()).collect();
    let s1 = &holder[holder.len() - 1];
    match create_folder(&format!("{}/{}", target, s1), comand) {
        Some(_) => {}
        None => return None,
    }
    let paths = match read_dir(s) {
        Ok(dir) => dir,
        Err(error) => {
            eprintln!("error read dir in move dir {:?}", error);
            let _ = remove_dir(&format!("{}/{}", target, s1));
            return None;
        }
    };

    for path in paths {
        let d = match path {
            Ok(d) => d,
            Err(error) => {
                eprintln!("error Dir entry in move dir{:?}", error);
                let _ = remove_dir(&format!("{}/{}", target, s1));
                return None;
            }
        };
        let d_path = d.path().to_string_lossy().to_string();
        let d_meta_data = match d.metadata() {
            Ok(data) => data,
            Err(error) => {
                eprintln!("error metadata in move dir{:?}", error);
                let _ = remove_dir(&format!("{}/{}", target, s1));
                return None;
            }
        };
        if d_meta_data.is_dir() {
            match move_dir(&d_path, &target, comand) {
                Some(_) => {}
                None => {
                    let _ = remove_dir(&format!("{}/{}", target, s1));
                    return None;
                }
            }
        } else {
            match move_file(&d_path, &format!("{}/{}", target, s1), comand) {
                Some(_) => {}
                None => {
                    let _ = remove_dir(&format!("{}/{}", target, s1));
                    return None;
                }
            }
        }
    }
    match remove_dir_all(s) {
        Ok(_) => Some(true),
        Err(e) => {
            eprintln!("is_exist mv error {:?}", e);
            let _ = remove_dir(&format!("{}/{}", target, s1));
            return None;
        }
    }
}

pub fn move_file(source: &String, target: &String, comand: &String) -> Option<bool> {
    let is_exist = match exists(target) {
        Ok(check) => check,
        Err(err) => {
            eprintln!("is_exist mv error {:?}", err);
            return None;
        }
    };
    if !is_exist {
        if target.ends_with("/") {
            eprintln!(
                "{}: rename {} to {}: No such file or directory",
                comand, source, target
            );
            return None;
        } else {
            match rename_file_or_move(source, target, comand) {
                Some(_) => return Some(true),
                None => return None,
            }
        }
    } else {
        match rename_file_or_move(source, target, comand) {
            Some(_) => return Some(true),
            None => return None,
        }
    }
}

pub fn rename_file_or_move(source: &String, target: &String, comand: &String) -> Option<bool> {
    let source_data = match metadata(source) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("{:?}", e);
            return None;
        }
    };
    if !demand_confirmation(source_data, source) {
        return None;
    }
    let content: Vec<u8> = match read(source) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("Error reading file: {} {}", source, error);
            return None;
        }
    };

    match create_and_remove(target, &content, source, comand) {
        Some(_) => return Some(true),
        None => return None,
    }
}

pub fn create_and_remove(
    target: &String,
    content: &Vec<u8>,
    source: &String,
    comand: &String,
) -> Option<bool> {
    let holder: Vec<String> = source.split("/").map(|f| f.to_string()).collect();
    let s = &holder[holder.len() - 1];
    match create_file(target, content, s, comand) {
        Some(path) => {
            copy_perms(source, &path);
            let _ = match remove_file(source) {
                Ok(_) => return Some(true),
                Err(err) => {
                    eprintln!("error in removing after rename a file {:?}", err);
                    return None;
                }
            };
        }
        None => return None,
    };
}
