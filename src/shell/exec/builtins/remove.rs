use crate::shell::{Shell, parse::Cmd};

use io::*;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::{
    fs::{self, Metadata, metadata, remove_dir_all, remove_file},
    io,
};
use users::{get_group_by_gid, get_user_by_uid};

pub fn rm(_shell: &mut Shell, command: &Cmd) {
    if command.args.len() == 0 {
        println!("usage: rm [-r] file ...\nunlink [--] file");
    }

    for path in &command.args {
        let is_exist = match fs::exists(path) {
            Ok(b) => b,
            Err(err) => {
                println!("{:?}", err);
                return;
            }
        };
        if is_exist {
            let data_of_target = match metadata(path) {
                Ok(data) => data,
                Err(err) => {
                    println!("{:?}", err);
                    return;
                }
            };
            if data_of_target.is_dir() {
                let flags: String = command.flags.iter().map(|c| c.to_string()).collect();
                if command.flags.len() == 0 {
                    println!("{}: {}: {}", command.exec, path, "is a directory");
                    return;
                } else if command.flags.len() > 1 || flags != "r" {
                    println!(
                        "{}: illegal option -- {}\nusage: rm [-r] file ...\nunlink [--] file",
                        command.exec, flags
                    );
                    return;
                } else {
                    if can_remove_directly(data_of_target, path) {
                        match remove_dir_all(path) {
                            Ok(_) => continue,
                            Err(error) => match error.kind() {
                                ErrorKind::PermissionDenied => {
                                    println!("{}: {}: {}", command.exec, path, "Permission denied")
                                }
                                _ => return,
                            },
                        };
                    }
                }
            } else {
                if can_remove_directly(data_of_target, path) {
                    let _ = remove_file(path);
                }
            }
        } else {
            println!(
                "{}: {}: {}",
                command.exec, path, "No such file or directory"
            );
        }
    }
}

pub fn can_remove_directly(data_of_target: Metadata, path: &String) -> bool {
    if data_of_target.permissions().mode() & 0o200 == 0 {
        let uid = data_of_target.uid();
        let gid = data_of_target.gid();
        let user_name = match get_user_by_uid(uid) {
            Some(user) => user,
            None => {
                println!("we can't get the user name");
                return false
            },
        };
        let group_name = match get_group_by_gid(gid) {

            Some(group) => group,
            None => {
                println!("we can't get the group name");
                return false;
            },
        };
        print!(
            "override r--r--r-- {}/{} for {}? ",
            user_name.name().to_string_lossy(),
            group_name.name().to_string_lossy(),
            path
        );
        io::stdout().flush().unwrap();
        //flush the buffer after the print

        let mut response: String = String::new();
        let _ = io::stdin().read_line(&mut response);

        let response = response.trim();
        if response.starts_with("y") || response.starts_with("Y") {
            return true;
        } else {
            return false;
        }
    } else {
        return true;
    }
}
