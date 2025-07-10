use crate::shell::{Shell, parse::Cmd};
use io::*;
use std::{fs::create_dir, io};

/*
    mkdir folder already existed =>  mkdir: Desktop: File exists
    mkdir folder existed and one not => mkdir: Desktop: File exists\n create the new one
    mkdir folder with not valid path => mkdir: jj: No such file or directory
*/

pub fn mkdir(_shell: &mut Shell, command: &Cmd) {
    println!("{:?}", command.args);
    for f in &command.args {
        let folder_name: &String = f;
        // let err : io::Error;
        create_dir(folder_name).unwrap_or_else(|error| {
            // err = error;
            match error.kind() {
                ErrorKind::NotFound => {
                    let not_found: Vec<&str> = f.split("/").collect();
                    println!(
                        "{}: {}: {}",
                        command.exec, not_found[0], "No such file or directory"
                    );
                }
                ErrorKind::AlreadyExists => {
                    println!("{}: {}: {}", command.exec, f, "File exists");
                }
                _ => println!("{}: {}", command.exec, error),
            }

            // err.clear();
        })
    }
}
