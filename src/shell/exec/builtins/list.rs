use super::*;

use chrono::{DateTime, Local};
use users::{get_group_by_gid, get_user_by_uid};

use std::{
    fs::{self, DirEntry, read_link, metadata},
    os::unix::fs::{MetadataExt, PermissionsExt},
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
                return match read_link(name.path()) {
                    Ok(link) => Types::Symlink(link.into_os_string()),
                    Err(_) => Types::Symlink(name.file_name()), 
                };
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

fn list_args(meta: &std::fs::Metadata) -> String {
    let mode = meta.permissions().mode();
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
fn get_group_and_user_meta(meta: &std::fs::Metadata) -> (String, String) {
    let uid = meta.uid();
    let gid = meta.gid();
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
fn get_time_meta(meta: &std::fs::Metadata) -> String {
    let time = match meta.modified() {
        Ok(mtime) => mtime + Duration::from_secs(3600),
        Err(_) => return "?".to_string(),
    };
    let now = SystemTime::now();
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
fn handle_show_entries(args: &Cmd) {
    let dot = std::fs::metadata(".");
    let dotdot = std::fs::metadata("..");
    if args.flags.contains(&"l".to_string()) {
        if let Ok(meta) = dot {
            let perms = list_args(&meta);
            let nlinks = meta.nlink();
            let (size_or_dev, _) = match perms.chars().next() {
                Some('c') | Some('b') => {
                    let rdev = meta.rdev();
                    let major = (rdev >> 8) & 0xfff;
                    let minor = (rdev & 0xff) | ((rdev >> 12) & 0xfff00);
                    (format!("{}, {}", major, minor), true)
                }
                _ => (format!("{}", meta.len()), false)
            };
            let date = get_time_meta(&meta);
            let (username, group) = get_group_and_user_meta(&meta);
            write_(&format!(
                "{} {} {} {} {:>5} {:>5} .\n",
                perms, nlinks, username, group, size_or_dev, date
            ));
        }
        if let Ok(meta) = dotdot {
            let perms = list_args(&meta);
            let nlinks = meta.nlink();
            let (size_or_dev, _) = match perms.chars().next() {
                Some('c') | Some('b') => {
                    let rdev = meta.rdev();
                    let major = (rdev >> 8) & 0xfff;
                    let minor = (rdev & 0xff) | ((rdev >> 12) & 0xfff00);
                    (format!("{}, {}", major, minor), true)
                }
                _ => (format!("{}", meta.len()), false)
            };
            let date = get_time_meta(&meta);
            let (username, group) = get_group_and_user_meta(&meta);
            write_(&format!(
                "{} {} {} {} {:>5} {:>5} ..\n",
                perms, nlinks, username, group, size_or_dev, date
            ));
        }
    } else {
        write_(". .. ");
    }
}

pub fn ls(_shell: &mut Shell, args: &Cmd) {
    let mut output = String::new();
    let show = args.flags.contains(&"a".to_string());
    let classify = args.flags.contains(&"F".to_string());
    let blue = "\x1b[34m";
    let green = "\x1b[32m";
    let reset = "\x1b[0m";

    let targets: Vec<String> = if args.args.is_empty() {
        vec![".".to_string()]
    } else {
        args.args.clone()
    };

    for target in targets {
        let meta = std::fs::metadata(&target);
        match meta {
            Ok(meta) => {
                if meta.is_dir() {
                    let readir = fs::read_dir(&target);
                    let mut entries: Vec<_> = match readir {
                        Ok(rd) => rd.filter_map(Result::ok).collect(),
                        Err(_) => {
                            write_(&format!("ls : cannot access {}: No such file or dir\n", target));
                            continue;
                        }
                    };
                    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
                    if args.flags.contains(&"l".to_string()) {
                        let mut total_blocks = 0;
                        let show_all = args.flags.contains(&"a".to_string());
                        if show_all {
                            if let Ok(meta) = metadata(&target) {
                                total_blocks += meta.blocks();
                            }
                            if let Ok(meta) = metadata(format!("{}/..", &target)) {
                                total_blocks += meta.blocks();
                            }
                        }
                        for entry in &entries {
                            let name = entry.file_name();
                            let name_str = name.to_string_lossy();
                            if !show_all && name_str.starts_with('.') {
                                continue;
                            }
                            if let Ok(meta) = entry.metadata() {
                                total_blocks += meta.blocks();
                            }
                        }
                        write_(&format!("total {}\n", total_blocks / 2));
                    }
                    if show {
                        handle_show_entries(args);
                    }
                    for elems in entries {
                        if args.flags.contains(&"l".to_string()) {
                            let perms = match elems.metadata() {
                                Ok(m) => list_args(&m),
                                Err(_) => String::from("?"),
                            };
                            let nlinks = match elems.metadata() {
                                Ok(meta) => meta.nlink(),
                                Err(_) => 0,
                            };
                            let (username, group) = get_group_and_user(&elems);
                            let (size_or_dev, _) = match perms.chars().next() {
                                Some('c') | Some('b') => {
                                    let rdev = elems.metadata().unwrap().rdev();
                                    let major = (rdev >> 8) & 0xfff;
                                    let minor = (rdev & 0xff) | ((rdev >> 12) & 0xfff00);
                                    (format!("{}, {}", major, minor), true)
                                }
                                _ => (format!("{}", elems.metadata().unwrap().len()), false)
                            };
                            let date = get_time(&elems);
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
                                    if classify {
                                        if show {
                                            output.push_str(&format!("{}*{}{}", green, name_str, reset));
                                        } else if !name_str.starts_with('.') {
                                            output.push_str(&format!("{}*{}{}", green, name_str, reset));
                                        }
                                    } else {
                                        if show {
                                            output.push_str(&format!("{}{}{}", green, name_str, reset));
                                        } else if !name_str.starts_with('.') {
                                            output.push_str(&format!("{}{}{}", green, name_str, reset));
                                        }
                                    }
                                }
                                Types::Symlink(name) => {
                                    let name_str = name.to_string_lossy();
                                    if args.flags.contains(&"l".to_string()) {
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
                                    } else {
                                        let display = if classify {
                                            format!("{}@", elems.file_name().to_string_lossy())
                                        } else {
                                            elems.file_name().to_string_lossy().to_string()
                                        };
                                        let colored = format!("{}{}{}", blue, display, reset);
                                        if show {
                                            output.push_str(&colored);
                                        } else if !display.starts_with('.') {
                                            output.push_str(&colored);
                                        }
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
                            if !output.is_empty() {
                                write_(&format!(
                                    "{} {} {} {} {:>5} {:>5} {}\n",
                                    perms, nlinks, username, group, size_or_dev, date, output
                                ));
                                output.clear();
                            }
                        } else {
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
                                    if classify {
                                        if show {
                                            output.push_str(&format!("{}*{}{}", green, name_str, reset));
                                        } else if !name_str.starts_with('.') {
                                            output.push_str(&format!("{}*{}{}", green, name_str, reset));
                                        }
                                    } else {
                                        if show {
                                            output.push_str(&format!("{}{}{}", green, name_str, reset));
                                        } else if !name_str.starts_with('.') {
                                            output.push_str(&format!("{}{}{}", green, name_str, reset));
                                        }
                                    }
                                }
                                Types::Symlink(name) => {
                                    let name_str = name.to_string_lossy();
                                    if args.flags.contains(&"l".to_string()) {
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
                                    } else {
                                        let display = if classify {
                                            format!("{}@", elems.file_name().to_string_lossy())
                                        } else {
                                            elems.file_name().to_string_lossy().to_string()
                                        };
                                        let colored = format!("{}{}{}", blue, display, reset);
                                        if show {
                                            output.push_str(&colored);
                                        } else if !display.starts_with('.') {
                                            output.push_str(&colored);
                                        }
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
                            if !output.is_empty() {
                                write_(&format!("{}\n", output));
                                output.clear();
                            }
                        }
                    }
                } else {
                    if args.flags.contains(&"l".to_string()) {
                        let perms = list_args(&meta);
                        let nlinks = meta.nlink();
                        let (size_or_dev, _) = match perms.chars().next() {
                            Some('c') | Some('b') => {
                                let rdev = meta.rdev();
                                let major = (rdev >> 8) & 0xfff;
                                let minor = (rdev & 0xff) | ((rdev >> 12) & 0xfff00);
                                (format!("{}, {}", major, minor), true)
                            }
                            _ => (format!("{}", meta.len()), false)
                        };
                        let date = get_time_meta(&meta);
                        let (username, group) = get_group_and_user_meta(&meta);
                        write_(&format!(
                            "{} {} {} {} {:>5} {:>5} {}\n",
                            perms, nlinks, username, group, size_or_dev, date, target
                        ));
                    } else {
                        write_(&format!("{}\n", target));
                    }
                }
            }
            Err(_) => {
                write_(&format!("ls : cannot access {}: No such file or dir\n", target));
            }
        }
    }
}
