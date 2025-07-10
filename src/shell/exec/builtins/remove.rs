use crate::shell::{exec::check_type, parse::Cmd, Shell};

use io::*;
use std::{fs::{self, metadata, remove_dir_all, remove_file, Metadata}, io};
use std::os::unix::fs::{MetadataExt,PermissionsExt};
use users::{get_user_by_uid, get_group_by_gid};


/*
    mkdir folder already existed =>  mkdir: Desktop: File exists
    mkdir folder existed and one not => mkdir: Desktop: File exists\n create the new one
    mkdir folder with not valid path => mkdir: jj: No such file or directory
*/

pub fn rm(_shell: &mut Shell, command: &Cmd) {
    for path in &command.args {
        let is_exist = match fs::exists(path) {
            Ok(b) => b,
            Err(err) => {
                println!("{:?}", err);
                return
            } ,
        };
        if is_exist {
            let data_of_target = match metadata(path) {
                Ok(data) => data,
                Err(err) => {
                    println!("{:?}", err);
                    return
                },
            };
            if data_of_target.is_dir() {
                if command.flags.len() != 1 || command.flags[0] != "r" {
                    let flags : String = command.flags.iter().map(|c| c.to_string()).collect();
                    println!("{}: illegal option -- {}\nusage: rm [-r] file ...\nunlink [--] file",command.exec,flags);
                }else {
                    if can_remove_directly(data_of_target, path) {
                        match remove_dir_all(path) {
                            Ok(_) => return,
                            Err(error) =>match error.kind() {
                                ErrorKind::PermissionDenied => println!("{}: {}: {}",command.exec, path, "Permission denied"),
                                _ => return,
                            },
                        };
                    }
                }
            }else {
                if can_remove_directly(data_of_target, path) {
                    let _ = remove_file(path);
                }
            }
        }else {
            println!("{}: {}: {}",command.exec, path, "No such file or directory");
        }
    }
}


pub fn can_remove_directly(data_of_target: Metadata, path: &String) -> bool {
    if data_of_target.permissions().mode() & 0o777 == 0o444 {
        let uid = data_of_target.uid();
        let gid = data_of_target.gid();
        let user_name = get_user_by_uid(uid).unwrap();
        let group_name = get_group_by_gid(gid).unwrap();
        print!("override r--r--r-- {}/{} for {}? ",user_name.name().to_string_lossy(), group_name.name().to_string_lossy() ,path);
        io::stdout().flush().unwrap();
        //flush the buffer after the print

        let mut response :String = String::new();
        let _ = io::stdin().read_line(&mut response);

        let response = response.trim();
        if response.starts_with("y")|| response.starts_with("Y") {
            return true;
        }else {
            return false;
        }
    } else {
        return true;
    }
}
