use super::*;

use chrono::{DateTime, Local};
use users::{get_group_by_gid, get_user_by_uid};

use std::{
    fs::{self, DirEntry, read_link},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
    time::{Duration, SystemTime},
};

#[derive(Debug)]
pub enum Types {
    File(OsString),
    Dir(OsString),
    Executable(OsString),
    Symlink(OsString),
    NotSupported,
    Error,
}

pub fn check_type(name: &DirEntry) -> Types {
    match name.metadata() {
        Ok(meta) => {
            if meta.is_symlink() {
                return Types::Symlink(read_link(name.path()).unwrap().into_os_string());
            } else if meta.permissions().mode() & 0o111 != 0 && meta.is_file() {
                return Types::Executable(name.file_name());
            } else if meta.is_file() {
                return Types::File(name.file_name());
            } else if meta.is_dir() {
                return Types::Dir(name.file_name());
            } else {
                return Types::NotSupported;
            }
        }
        _ => Types::Error,
    }
}

pub fn list_arg(args: &DirEntry) -> String {
    let mode = match args.metadata() {
        Ok(meta) => meta.permissions().mode(),
        Err(_) => std::process::exit(1),
    };
    let file_type = match mode & 0o170000 {
        0o040000 => 'd', // directory
        0o100000 => '-', // regular file
        0o120000 => 'l', // symlink
        0o140000 => 's', //socket
        0o010000 => 'p', //pipe
        0o060000 => 'b', //disc
        0o020000 => 'c', //keyb , tty, ms
        _ => '?',        // other
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
    // setuid
    perms.push(match (mode & 0o100 != 0, suid) {
        (true, true) => 's',
        (false, true) => 'S',
        (true, false) => 'x',
        (false, false) => '-',
    });

    // Group permissions
    perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    //setguid
    perms.push(match (mode & 0o010 != 0, sgid) {
        (true, true) => 's',
        (false, true) => 'S',
        (true, false) => 'x',
        (false, false) => '-',
    });

    // Others permissions
    perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    // sticky bit
    perms.push(match (mode & 0o001 != 0, sticky) {
        (true, true) => 't',
        (false, true) => 'T',
        (true, false) => 'x',
        (false, false) => '-',
    });
    perms
}

pub fn get_group_and_user(args: &DirEntry) -> (String, String) {
    let uid = match args.metadata() {
        Ok(meta) => meta.uid(),
        Err(_) => std::process::exit(1),
    };
    let gid = match args.metadata() {
        Ok(meta) => meta.gid(),
        Err(_) => std::process::exit(1),
    };
    let username = match get_user_by_uid(uid) {
        Some(u) => u.name().to_string_lossy().to_string(),
        None => "None".to_string(),
    };
    let group = match get_group_by_gid(gid) {
        Some(g) => g.name().to_string_lossy().to_string(),
        _ => "None".to_string(),
    };
    (username, group)
}
pub fn get_time(args: &DirEntry) -> String {
    let time = match args.metadata().and_then(|m| m.modified()) {
        Ok(mtime) => mtime + Duration::from_secs(3600),
        Err(_) => return "?".to_string(),
    };
    let now = SystemTime::now();
    // println!("time {:#?} ", now);
    let under_six = Duration::from_secs(60 * 60 * 24 * 30);
    let passed = match now.duration_since(time) {
        Ok(duration) => duration < under_six,
        Err(_) => true,
    };
    let formated: DateTime<Local> = time.into();

    if passed {
        formated.format("%b %e %H:%M").to_string()
    } else {
        formated.format("%b %e  %Y").to_string()
    }
}
pub fn ls(_shell: &mut Shell, args: &Cmd) {
    let mut paths = Vec::new();
    if args.args.is_empty() {
        paths.push(fs::read_dir("."));
    } else {
        for item in &args.args {
            paths.push(fs::read_dir(item));
        }
    }
    let mut output = String::new();
    let show = args.flags.contains(&"a".to_string());
    let classify = args.flags.contains(&"F".to_string());
    let mut perms = String::new();
    let mut username = String::new();
    let mut group = String::new();
    let mut nlinks = 0;
    let mut size = 0 as u64;
    let mut date = String::new();
    let blue = "\x1b[34m";
    let green = "\x1b[32m";
    let reset = "\x1b[0m";
    if show {
        paths.push(PathBuf::from("."));
        paths.push(PathBuf::from(".."));
    }

    for data in paths {
        let readir = match data {
            Ok(v) => v,
            _ => {
                write_("ls : cannot access : No such file or dir\n");
                continue;
            }
        };
        let mut entries: Vec<_> = readir.filter_map(Result::ok).collect();
        entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        for elems in entries {
            if args.flags.contains(&"l".to_string()) {
                perms = list_arg(&elems);
                nlinks = match elems.metadata() {
                    Ok(meta) => meta.nlink(),
                    Err(_) => 0,
                };
                (username, group) = get_group_and_user(&elems);
                size = match elems.metadata() {
                    Ok(meta) => meta.len(),
                    Err(_) => 0,
                };
                date = get_time(&elems);
            }
            match check_type(&elems) {
                Types::Dir(name) => {
                    let name_str = name.to_string_lossy();
                    let display = if classify {
                        format!("{}/", name_str)
                    } else {
                        name_str.to_string()
                    };
                    let colored = format!("{}{}{}", blue, display, reset);
                    if show {
                        output.push_str(&colored);
                    } else if !name_str.starts_with('.') {
                        output.push_str(&colored);
                    }
                }
                Types::Executable(name) => {
                    let name_str = name.to_string_lossy();
                    if show {
                        output.push_str(&format!("{}{}{}", green, name_str, reset));
                    } else if !name_str.starts_with('.') {
                        output.push_str(&format!("{}{}{}", green, name_str, reset));
                    }
                }

                Types::Symlink(name) => {
                    let name_str = name.to_string_lossy();
                    if show {
                        output.push_str(&format!(
                            "{} -> {}",
                            elems.path().to_string_lossy(),
                            name_str
                        ));
                    } else if !name_str.starts_with('.') {
                        output.push_str(&format!(
                            "{} -> {}",
                            elems.path().to_string_lossy(),
                            name_str
                        ));
                    }
                }
                Types::File(name) => {
                    let name_str = name.to_string_lossy();
                    if show {
                        output.push_str(&name_str);
                    } else if !name_str.starts_with('.') {
                        output.push_str(&name_str);
                    }
                }
                _ => (),
            }
            if args.flags.contains(&"l".to_string()) && !output.is_empty() {
                write_(&format!(
                    "{} {} {} {} {:>5} {:>5} {}\n",
                    perms, nlinks, username, group, size, date, output
                ));
                output.clear();
            } else if !output.is_empty() {
                write_(&format!("{}\n", output));
                output.clear();
            }
        }
    }
}
