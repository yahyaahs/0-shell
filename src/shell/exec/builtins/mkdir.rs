use crate::shell::{Shell, parse::Cmd};
use std::{fs::create_dir, io, path};
use io::*;
/*
    mkdir folder already existed =>  mkdir: Desktop: File exists
    mkdir folder existed and one not => mkdir: Desktop: File exists\n create the new one
    mkdir folder with not valid path => mkdir: jj: No such file or directory
*/
pub fn mkdir(shell: &mut Shell, command:&Cmd) {
    println!("{:?}",command.args);
    let folder_name : &String = &command.args[0];
    // let err : io::Error;
    create_dir(folder_name).unwrap_or_else(|error|{
        // err = error;
        match error.kind() {
            ErrorKind::NotFound => {
                let not_found : Vec<&str>= command.args[0].split("/").collect();
                println!("{}: {}: {}",command.exec,not_found[0],"No such file or directory");
            },
            ErrorKind::AlreadyExists => {
                let already_exist : Vec<&str>= command.args[0].split("/").collect();
                println!("{}: {}: {}",command.exec,already_exist[0],"File exists");

            }
            _ => println!("{}: {}",command.exec,error)
        }
       
        // err.clear();
    })
}