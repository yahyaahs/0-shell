use super::*;

use std::{
    env,
    fs::{self, metadata, remove_dir_all, remove_file},
    io::*,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
};

use users::{get_group_by_gid, get_user_by_uid};

pub fn rm(_shell: &mut Shell, command: &Cmd) {
    if command.args.len() == 0 {
        eprintln!("usage: rm [-r] file ...\nunlink [--] file");
        return;
    }

    for mut path in command.args.clone() {
        if path.starts_with("~") {
            let home = env::var("HOME");
            match home {
                Ok(p) => path = path.replace("~", &p),
                Err(_) => {
                    write_("rm: cannot find HOME directory set\n");
                    return;
                }
            }
        }
        if path.contains("./") || path.contains("../") {
            eprintln!("rm: {:?} and {:?} may not be removed", ".", "..");
            return;
        }

        let is_symlink = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata.is_symlink(),
            Err(_) => false,
        };

        if is_symlink {
            let symlink_meta = match fs::symlink_metadata(&path) {
                Ok(meta) => meta,
                Err(err) => {
                    eprintln!("{:?}", err.to_string());
                    return;
                }
            };

            if demand_confirmation(symlink_meta, &path) {
                match remove_file(&path) {
                    Ok(_) => continue,
                    Err(error) => match error.kind() {
                        ErrorKind::PermissionDenied => {
                            eprintln!("{}: {}: {}", command.exec, path, "Permission denied");
                            return;
                        }
                        _ => {
                            eprintln!("{}: {}: {}", command.exec, path, error);
                            return;
                        }
                    },
                }
            }
            continue;
        }

        let is_exist = match fs::exists(&path) {
            Ok(b) => b,
            Err(err) => {
                eprintln!("{:?}", err);
                return;
            }
        };
        if is_exist {
            let data_of_source = match metadata(&path) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("{:?}", err);
                    return;
                }
            };
            if data_of_source.is_dir() {
                let flags: String = command.flags.iter().map(|c| c.to_string()).collect();
                if command.flags.len() == 0 {
                    eprintln!("{}: {}: {}", command.exec, path, "is a directory");
                    return;
                } else if command.flags.len() > 1 || flags != "r" {
                    eprintln!(
                        "{}: illegal option -- {}\nusage: rm [-r] file ...\nunlink [--] file",
                        command.exec, flags
                    );
                    return;
                } else {
                    if demand_confirmation(data_of_source, &path) {
                        match remove_dir_all(&path) {
                            Ok(_) => continue,
                            Err(error) => match error.kind() {
                                ErrorKind::PermissionDenied => {
                                    eprintln!(
                                        "{}: {}: {}",
                                        command.exec, path, "Permission denied"
                                    );
                                    return;
                                }
                                _ => return,
                            },
                        };
                    }
                }
            } else {
                if demand_confirmation(data_of_source, &path) {
                    let _ = remove_file(path);
                }
            }
        } else {
            let p = match Path::new(&path).canonicalize() {
                Ok(p) => p,
                Err(e) => {
                    println!("aaaa {}", e.to_string());
                    return;
                }
            };
            println!("{:?}", p);
            eprintln!(
                "{}: {}: {}",
                command.exec,
                path.clone(),
                "hhhh No such file or directory"
            );
        }
    }
}

pub fn demand_confirmation(data_of_source: fs::Metadata, path: &String) -> bool {
    if data_of_source.permissions().mode() & 0o200 == 0 {
        let uid = data_of_source.uid();
        let gid = data_of_source.gid();
        let user_name = match get_user_by_uid(uid) {
            Some(user) => user,
            None => {
                eprintln!("we can't get the user name");
                return false;
            }
        };
        let group_name = match get_group_by_gid(gid) {
            Some(group) => group,
            None => {
                eprintln!("we can't get the group name");
                return false;
            }
        };
        let pers = list_args(&data_of_source);
        write_(&format!(
            "override {} {}/{} for {}? ",
            pers,
            user_name.name().to_string_lossy(),
            group_name.name().to_string_lossy(),
            path
        ));

        let mut response: String = String::new();
        let error = io::stdin().read_line(&mut response);
        if let Err(_) = error {
            return false;
        }

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

pub fn list_args(meta: &std::fs::Metadata) -> String {
    let mode = meta.permissions().mode();
    let file_type = match mode & 0o170000 {
        0o040000 => 'd',
        0o100000 => '-',
        0o120000 => 'l',
        0o140000 => 's',
        0o010000 => 'p',
        0o060000 => 'b',
        0o020000 => 'c',
        _ => '?',
    };
    let mut perms = String::new();
    perms.push(file_type);
    // Special
    let suid = mode & 0o4000 != 0;
    let sgid = mode & 0o2000 != 0;
    let sticky = mode & 0o1000 != 0;
    // User permissions
    perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    perms.push(match (mode & 0o100 != 0, suid) {
        (true, true) => 's',
        (false, true) => 'S',
        (true, false) => 'x',
        (false, false) => '-',
    });
    // Group permissions
    perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    perms.push(match (mode & 0o010 != 0, sgid) {
        (true, true) => 's',
        (false, true) => 'S',
        (true, false) => 'x',
        (false, false) => '-',
    });
    // Others permissions
    perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    perms.push(match (mode & 0o001 != 0, sticky) {
        (true, true) => 't',
        (false, true) => 'T',
        (true, false) => 'x',
        (false, false) => '-',
    });
    perms
}
