use super::*;

use std::{fs::create_dir, io::*};

pub fn mkdir(_shell: &mut Shell, command: &Cmd) {
    for folder_name in &command.args {
        match create_folder(folder_name, &command.exec) {
            Some(_) => {},
            None => return
        }
    }
}

pub fn create_folder(folder_name: &String, comand: &String) -> Option<bool>{
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
            },
            ErrorKind::PermissionDenied => {
                eprintln!("{}: {}: {}", comand, folder_name, "Permission denied");
                None
            }
            _ => {
                eprintln!("hanni {}: {}", comand, error);
                None
            }
        }
    }
}
