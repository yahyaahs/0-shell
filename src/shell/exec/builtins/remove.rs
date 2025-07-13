use super::*;

use io::*;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::{
    fs::{self, Metadata, metadata, remove_dir_all, remove_file},
    io,
};
use users::{get_group_by_gid, get_user_by_uid};

pub fn rm(_shell: &mut Shell, command: &Cmd) {
    if command.args.len() == 0 {
        write_("usage: rm [-r] file ...\nunlink [--] file\n");
    }

    for path in &command.args {
        let is_exist = match fs::exists(path) {
            Ok(b) => b,
            Err(err) => {
                write_(&format!("{:?}\n", err));
                return;
            }
        };
        if is_exist {
            let data_of_target = match metadata(path) {
                Ok(data) => data,
                Err(err) => {
                    write_(&format!("{:?}\n", err));
                    return;
                }
            };
            if data_of_target.is_dir() {
                let flags: String = command.flags.iter().map(|c| c.to_string()).collect();
                if command.flags.len() == 0 {
                    write_(&format!("{}: {}: {}\n", command.exec, path, "is a directory"));
                    return;
                } else if command.flags.len() > 1 || flags != "r" {
                    write_(&format!(
                        "{}: illegal option -- {}\nusage: rm [-r] file ...\nunlink [--] file\n",
                        command.exec, flags
                    ));
                    return;
                } else {
                    if can_remove_directly(data_of_target, path) {
                        match remove_dir_all(path) {
                            Ok(_) => return,
                            Err(error) => match error.kind() {
                                ErrorKind::PermissionDenied => {
                                    write_(&format!("{}: {}: {}\n", command.exec, path, "Permission denied"))
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
            write_(&format!(
                "{}: {}: {}\n",
                command.exec, path, "No such file or directory"
            ));
        }
    }
}

pub fn can_remove_directly(data_of_target: Metadata, path: &String) -> bool {
    if data_of_target.permissions().mode() & 0o777 == 0o444 {
        let uid = data_of_target.uid();
        let gid = data_of_target.gid();
        let user_name = get_user_by_uid(uid).unwrap();
        let group_name = get_group_by_gid(gid).unwrap();
        write_(&format!(
            "override r--r--r-- {}/{} for {}? ",
            user_name.name().to_string_lossy(),
            group_name.name().to_string_lossy(),
            path
        ));
        io::stdout().flush().unwrap();

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
