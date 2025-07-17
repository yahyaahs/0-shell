use super::*;

use std::{
    env,
    fs::{self, metadata},
    io::*,
    os::unix::fs::PermissionsExt,
};

pub fn cp(_shell: &mut Shell, command: &Cmd) {
    if command.args.len() < 2 {
        eprintln!("usage: cp source_file target_file\n       cp source_file ... target_directory");
        return;
    }
    let sources = command.args[0..command.args.len() - 1].to_vec();
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
    if sources.len() == 1 {
        let mut only_src = sources[0].clone();
        if only_src.starts_with("~") {
            let home = env::var("HOME");
            match home {
                Ok(p) => only_src = only_src.replace("~", &p),
                Err(_) => {
                    write_("cd: cannot find HOME directory set\n");
                    return;
                }
            }
        }
        one_source(&only_src, &command.exec, &target);
    } else {
        let data_of_target = match metadata(&target) {
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
        for mut source in sources {
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
            let data_of_source = match metadata(&source) {
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
            let content: Vec<u8> = match fs::read(&source) {
                Ok(data) => {
                    data
                }
                Err(error) => {
                    eprintln!("Error reading file: {}", error);
                    return;
                }
            };
            match create_file(&target, &content, &source, &command.exec) {
                Some(path) => {
                    if path == "".to_string() {
                        return;
                    }

                    copy_perms(&source, &path);
                }
                None => return,
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
    let content: Vec<u8> = match fs::read(source) {
        Ok(data) => {
            data
        }
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
            if path == "".to_string() {
                return;
            }

            copy_perms(source, &path);
        }
        None => return,
    }
}

pub fn create_file(
    path: &String,
    content: &Vec<u8>,
    source: &String,
    command: &String,
) -> Option<String> {
    let s: Vec<String> = source.split("/").map(|f| f.to_string()).collect();
    let s1 = &s[s.len() - 1];
    let mut new_path = path;
    let holder = &format!("{}/{}", path, s1);
    match fs::write(path, content) {
        Ok(_) => Some(path.clone()),
        Err(error) => match error.kind() {
            ErrorKind::IsADirectory => {
                if path.ends_with(s1) {
                    eprintln!(
                        "cp: cannot overwrite directory {} with non-directory {}",
                        format!("{}/{}", path, s1),
                        source
                    );
                    return None;
                }
                new_path = holder;

                match create_file(new_path, content, source, command) {
                    Some(_) => Some(new_path.clone()),
                    None => return None,
                }
            }
            ErrorKind::PermissionDenied => {
                eprintln!("{}: {}: {}", command, path, "Permission denied");
                return None;
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
                eprintln!("{}: {}: {}", command, path, "Not Found");
                return None;
            }
            _ => {
                eprintln!("{}", error);
                return None;
            }
        },
    };
    Some(new_path.clone())
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
