use std::{fs::{exists, metadata, read_dir, read_to_string, remove_dir_all, remove_file}};


use crate::shell::exec::builtins::{copy::{create_file, copy_perms}, mkdir::create_folder};

use super::*;
pub fn mv(_:&mut Shell, command: &Cmd) {
    let sources = &command.args[..command.args.len()-1].to_vec();
    let target =  &command.args[command.args.len() -1];
    for source in sources {
        let is_exist = match exists(source) {
            Ok(check) => check,
            Err(err) => {
                write_(&format!("is_exist mv error {:?}",err)); 
                return
            }
        };
        if !is_exist {
            write_(&format!("{}: rename {} to {}: No such file or directory", command.exec, source, target));
            continue
        }else {
            let meta_data_source = match metadata(source) {
                Ok(data) => data,
                Err(err) => {write_(&format!("metadata source mv error {:?}",err)); return}
            };
            
            if meta_data_source.is_dir() {
                move_dir(source, target, &command.exec);
            }else {
                move_file(source, target, &command.exec)
            }
        }
    }
}

pub fn move_dir(s: &String, target: &String, comand: &String) {

    let target_meta_data = match metadata(target) {
        Ok(data) => data,
        Err(error) => {
            write_(&format!("error metadata in move dir{:?}", error));
            return
        },
    };
    if !target_meta_data.is_dir() {
        write_(&format!("{}: rename {} to {}: Not a directory",comand, s, target));
        return
    }

        let holder : Vec<String> = s.split("/").map(|f| f.to_string()).collect();
        let s1 = &holder[holder.len() -1];
        println!("creation folder {}",format!("{}/{}",target, s1));
        create_folder(&format!("{}/{}",target, s1), comand);
    let paths = match read_dir(s) {
        Ok(dir) => dir,
        Err(error) => {
            write_(&format!("error read dir in move dir {:?}", error));
            return
        },
    };

    for path in paths {
        let d = match path {
            Ok(d) => d,
            Err(error) => {
                write_(&format!("error Dir entry in move dir{:?}", error));
                return
            },
        };
        let d_path = d.path().to_string_lossy().to_string();
        let d_meta_data = match d.metadata() {
            Ok(data) => data,
            Err(error) => {
                write_(&format!("error metadata in move dir{:?}", error));
                return
            },
        };
        if d_meta_data.is_dir() {
            move_dir(&d_path, &target, comand)
        }else{
            move_file(&d_path, &format!("{}/{}",target, s1), comand)
        }
    }
    match remove_dir_all(s) {
        Ok(_) => {},
        Err(e) =>  {write_(&format!("is_exist mv error {:?}",e)); return}
    }

}

pub fn move_file(source: &String, target: &String, comand: &String) {
    let is_exist = match exists(target) {
        Ok(check) => check,
        Err(err) => {write_(&format!("is_exist mv error {:?}",err)); return}
    };
    if !is_exist {
        if target.ends_with("/") {
            write_(&format!("{}: rename {} to {}: No such file or directory",comand, source, target))
        }else{
            rename_file_or_move(source, target, comand);
        }
    }else {
        rename_file_or_move(source, target, comand);
    }
}

pub fn rename_file_or_move(source: &String, target: &String, comand: &String) {
    let content : String = match read_to_string(source) {
        Ok(data) => {
           data
        },
        Err(error) => {
            eprintln!("Error reading file: {} {}", source, error);
            return
        }
    };
    
    create_and_remove(target, &content, source, comand);
}

pub fn create_and_remove(target: &String, content: &String, source: &String, comand: &String) {
    let holder : Vec<String> = source.split("/").map(|f| f.to_string()).collect();
    let s = &holder[holder.len() -1];
    match create_file(target, &content, s, comand) {
        res => {
            if res.is_empty() {
                return
            }else {
                copy_perms(source, &res);
                let _ = match remove_file(source) {
                    Ok(_) => {},
                    Err(err) => {
                        write_(&format!("error in removing after rename a file {:?}", err));
                        return
                    }
                };
            }
        }
    }
}
