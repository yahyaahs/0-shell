use crate::shell::{Shell, parse::Cmd};
use io::*;
use std::{fs::{self, metadata, OpenOptions}, io};

pub fn cp(_shell: &mut Shell, command: &Cmd) {
    if command.args.len() < 2 {
        println!("usage: cp source_file target_file\n       cp source_file ... target_directory");
        return
    }
    println!("{:?}", command);
    let sources = command.args[0..command.args.len()-1].to_vec();
    let target = &command.args[command.args.len()-1];
    if sources.len() == 1 {
        one_source(&sources[0], &command.exec, target);
    }else {
        let data_of_target = match metadata(target) {
            Ok(data) => data,
            Err(error) => {println!("{:?}", error); return},
        };
        if !data_of_target.is_dir() {
            println!("{}: {} {}", command.exec, target, "is not a directory");
            return
        }
        for source in &sources {
            let data_of_file = match metadata(source) {
                Ok(data) => data,
                Err(error) => {println!("{:?}", error); return},
            };
            if data_of_file.is_dir() {
                println!("{}: {} {}", command.exec, source, "is a directory (not copied).");
                continue
            }
            let content : String = match fs::read_to_string(source) {
                Ok(data) => {
                   data
                },
                Err(error) => {
                    eprintln!("Error reading file: {}", error);
                    return
                }
            }; 
            create_file(target, &content, source);
        }
    }
}

pub fn one_source(source: &String, command: &String, target: &String) {
    if source == target {
        println!("{}: {} and {} {}",command, source, target, "are identical (not copied).");
        return
    }
    let data_of_source = match metadata(&source) {
        Ok(data) => data,
        Err(error) => {println!("{:?}", error); return},
    };
    if data_of_source.is_dir() {
        println!("{}: {} {}", command, source, "is a directory (not copied).");
        return
    }
    // println!("{:?}", data_of_source);
    // let is_exist = match fs::exists(target) {
    //     Ok(b) => b,
    //     Err(err) => {
    //         println!("{:?}", err);
    //         return;
    //     }
    // };
    // println!("{:?}",is_exist);
    // let data_of_target = match metadata(target) {
    //     Ok(data) => data,
    //     Err(error) => match error.kind() {
    //         ErrorKind::NotFound => {create_file(target); return},
    //         _ => return
    //     },
    // };
    // println!("data_of_target {:?}", data_of_target);
    let content : String = match fs::read_to_string(source) {
        Ok(data) => {
           data
        },
        Err(error) => {
            eprintln!("Error reading file: {}", error);
            return
        }
    }; 
    create_file(target, &content, source);
}

pub fn create_file(path: &String, content: &String, source: &String) {
    match fs::write(path, content) {
        Ok(_) => return,
        Err(error) => match error.kind() {
            ErrorKind::IsADirectory => {
                let new_path = format!("{}/{}",path,source);
                let _ = fs::write(new_path, content);
            }
            _ => {
                println!("{:?}", error);
            }
        }
    }
}
