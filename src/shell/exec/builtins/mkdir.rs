use crate::shell::{Shell, parse::Cmd};
use std::{fs::create_dir, io};
use io::*;
/*
    mkdir folder already existed =>  mkdir: Desktop: File exists
    mkdir folder existed and one not => mkdir: Desktop: File exists\n create the new one
    mkdir folder with not valid path => mkdir: jj: No such file or directory
*/
pub fn mkdir(_shell: &mut Shell, command:&Cmd) {
    if command.args.len() == 0 {
        println!("usage: mkdir directory_name ...");
        return
    };
    for f in &command.args {
        let folder_name : &String = f;
        create_dir(folder_name).unwrap_or_else(|error|{
            match error.kind() {
                ErrorKind::NotFound => {
                    let not_found : Vec<&str>= f.split("/").collect();
                    println!("{}: {}: {}",command.exec,not_found[0],"No such file or directory");
                    return
                },
                ErrorKind::AlreadyExists => {
                    let already_exist : Vec<&str>= f.split("/").collect();
                    println!("{}: {}: {}",command.exec,already_exist[0],"File exists");
                    return
                }
                _ => return
            }
        })
    }
   
}