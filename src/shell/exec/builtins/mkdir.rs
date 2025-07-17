use super::*;

use std::{env, fs::create_dir, io::*};

pub fn mkdir(_shell: &mut Shell, command: &Cmd) {
    for mut folder_name in command.args.clone() {
        if folder_name.starts_with("~") {
            let home = env::var("HOME");
            match home {
                Ok(p) => folder_name = folder_name.replace("~", &p),
                Err(_) => {
                    write_("cd: cannot find HOME directory set\n");
                    return;
                }
            }
        }
        match create_folder(&folder_name, &command.exec) {
            Some(_) => {}
            None => return,
        }
    }
}

pub fn create_folder(folder_name: &String, comand: &String) -> Option<bool> {
    match create_dir(folder_name) {
        Ok(_) => Some(true),
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                eprintln!(
                    "{}: {}: {}",
                    comand, folder_name, "No such file or directory"
                );
                None
            }
            ErrorKind::AlreadyExists => {
                eprintln!("{}: {}: {}", comand, folder_name, "File exists");
                None
            }
            ErrorKind::PermissionDenied => {
                eprintln!("{}: {}: {}", comand, folder_name, "Permission denied");
                None
            }
            _ => {
                eprintln!("hanni {}: {}", comand, error);
                None
            }
        },
    }
}
