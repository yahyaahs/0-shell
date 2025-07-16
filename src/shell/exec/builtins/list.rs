use super::*;

use chrono::{DateTime, Local};
use users::{get_group_by_gid, get_user_by_uid};
use std::os::unix::fs::FileTypeExt;
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
    CharDevice(OsString),
    BlockDevice(OsString),
    Socket(OsString),
    Pipe(OsString),
    NotSupported,
    Error,
}

pub fn check_type(name: &DirEntry) -> Types {
    match name.metadata() {
        Ok(meta) => {
            let ft = meta.file_type();
            if ft.is_symlink() {
                return Types::Symlink(read_link(name.path()).unwrap().into_os_string());
            } else if ft.is_char_device() {
                return Types::CharDevice(name.file_name());
            } else if ft.is_block_device() {
                return Types::BlockDevice(name.file_name());
            } else if ft.is_socket() {
                return Types::Socket(name.file_name());
            } else if ft.is_fifo() {
                return Types::Pipe(name.file_name());
            } else if meta.permissions().mode() & 0o111 != 0 && ft.is_file() {
                return Types::Executable(name.file_name());
            } else if ft.is_file() {
                return Types::File(name.file_name());
            } else if ft.is_dir() {
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
            let size = meta.len();
            let date = get_time_meta(&meta);
            let (username, group) = get_group_and_user_meta(&meta);
            write_(&format!(
                "{} {} {} {} {:>5} {:>5} .\n",
                perms, nlinks, username, group, size, date
            ));
        }
        if let Ok(meta) = dotdot {
            let perms = list_args(&meta);
            let nlinks = meta.nlink();
            let size = meta.len();
            let date = get_time_meta(&meta);
            let (username, group) = get_group_and_user_meta(&meta);
            write_(&format!(
                "{} {} {} {} {:>5} {:>5} ..\n",
                perms, nlinks, username, group, size, date
            ));
        }
    } else {
        write_(". .. ");
    }
}

fn major(dev: u64) -> u64 {
    (dev >> 8) & 0xfff
}
fn minor(dev: u64) -> u64 {
    (dev & 0xff) | ((dev >> 12) & 0xfff00)
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
                            let (size_str, _is_device) = match elems.metadata() {
                                Ok(meta) => {
                                    let file_type = meta.file_type();
                                    if file_type.is_char_device() || file_type.is_block_device() {
                                        let rdev = meta.rdev();
                                        (format!("{}, {}", major(rdev), minor(rdev)), true)
                                    } else {
                                        (format!("{}", meta.len()), false)
                                    }
                                }
                                Err(_) => ("?".to_string(), false),
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
                                    if show {
                                        output.push_str(&format!("{}{}{}", green, name_str, reset));
                                    } else if !name_str.starts_with('.') {
                                        output.push_str(&format!("{}{}{}", green, name_str, reset));
                                    }
                                }
                                Types::Symlink(_name) => {
                                    let file_name_os = elems.file_name();
                                    let file_name = file_name_os.to_string_lossy();
                                    let target = read_link(elems.path()).unwrap_or_default().to_string_lossy().to_string();
                                    let display = format!("{} -> {}", file_name, target);
                                    if show {
                                        output.push_str(&display);
                                    } else if !file_name.starts_with('.') {
                                        output.push_str(&display);
                                    }
                                }
                                Types::File(name)
                                | Types::CharDevice(name)
                                | Types::BlockDevice(name)
                                | Types::Socket(name)
                                | Types::Pipe(name) => {
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
                                    perms, nlinks, username, group, size_str, date, output
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
                                        output.push(' ');
                                    } else if !name_str.starts_with('.') {
                                        output.push_str(&colored);
                                        output.push(' ');
                                    }
                                }
                                Types::Executable(name) => {
                                    let name_str = name.to_string_lossy();
                                    if show {
                                        output.push_str(&format!("{}{}{} ", green, name_str, reset));
                                    } else if !name_str.starts_with('.') {
                                        output.push_str(&format!("{}{}{} ", green, name_str, reset));
                                    }
                                }
                                Types::Symlink(_name) => {
                                    let file_name_os = elems.file_name();
                                    let file_name = file_name_os.to_string_lossy();
                                    let target = read_link(elems.path()).unwrap_or_default().to_string_lossy().to_string();
                                    let display = format!("{} -> {} ", file_name, target);
                                    if show {
                                        output.push_str(&display);
                                    } else if !file_name.starts_with('.') {
                                        output.push_str(&display);
                                    }
                                }
                                Types::File(name)
                                | Types::CharDevice(name)
                                | Types::BlockDevice(name)
                                | Types::Socket(name)
                                | Types::Pipe(name) => {
                                    let name_str = name.to_string_lossy();
                                    if show {
                                        output.push_str(&format!("{} ", name_str));
                                    } else if !name_str.starts_with('.') {
                                        output.push_str(&format!("{} ", name_str));
                                    }
                                }
                                _ => (),
                            }
                        }
                        if !output.is_empty() && !args.flags.contains(&"l".to_string()) {
                            write_(&format!("{}\n", output));
                            output.clear();
                        }
                    }
                } else {
                    if args.flags.contains(&"l".to_string()) {
                        let perms = list_args(&meta);
                        let nlinks = meta.nlink();
                        let (size, _is_device) = {
                            let file_type = meta.file_type();
                            if file_type.is_char_device() || file_type.is_block_device() {
                                let rdev = meta.rdev();
                                (format!("{}, {}", major(rdev), minor(rdev)), true)
                            } else {
                                (format!("{}", meta.len()), false)
                            }
                        };
                        let date = get_time_meta(&meta);
                        let (username, group) = get_group_and_user_meta(&meta);
                        write_(&format!(
                            "{} {} {} {} {:>5} {:>5} {}\n",
                            perms, nlinks, username, group, size, date, target
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
