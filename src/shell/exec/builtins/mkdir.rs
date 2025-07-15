use super::*;

use std::{fs::create_dir, io::*};

pub fn mkdir(_shell: &mut Shell, command: &Cmd) {
    for folder_name in &command.args {
        create_folder(folder_name, &command.exec)
    }
}

pub fn create_folder(folder_name: &String, comand: &String) {
    create_dir(folder_name).unwrap_or_else(|error| match error.kind() {
        ErrorKind::NotFound => {
            println!(
                "{}: {}: {}",
                comand, folder_name, "No such file or directory"
            );
        }
        ErrorKind::AlreadyExists => {
            println!("{}: {}: {}", comand, folder_name, "File exists");
        }
        _ => println!("{}: {}", comand, error),
    })
}
