use super::*;

use std::{fs::create_dir, io::ErrorKind};

pub fn mkdir(_shell: &mut Shell, command: &Cmd) {
    for f in &command.args {
        let folder_name: &String = f;
        create_dir(folder_name).unwrap_or_else(|error| match error.kind() {
            ErrorKind::NotFound => {
                let not_found: Vec<&str> = f.split("/").collect();
                write_(&format!(
                    "{}: {}: {}\n",
                    command.exec, not_found[0], "No such file or directory"
                ));
            }
            ErrorKind::AlreadyExists => {
                let already_exist: Vec<&str> = f.split("/").collect();
                write_(&format!("{}: {}: {}\n", command.exec, already_exist[0], "File exists"));
            }
            _ => write_(&format!("{}: {}\n", command.exec, error)),
        })
    }
}
