use super::*;

use chrono::{DateTime, Local};
use users::{get_group_by_gid, get_user_by_uid};
use std::os::unix::fs::FileTypeExt;
use std::{
    fs::{self, DirEntry, read_link, metadata},
    os::unix::fs::{MetadataExt, PermissionsExt},
    time::{Duration, SystemTime},
};
use xattr::FileExt;
use std::fs::File;
use std::path::Path;

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
                return Types::Symlink(read_link(name.path()).unwrap_or_default().into_os_string());
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

fn has_acl(path: &std::path::Path) -> std::io::Result<bool> {
    let file = match File::open(path){
        Ok(file) => file,
        Err(_) => return Ok(false),
    };
    let acl_attrs = ["system.posix_acl_access", "system.posix_acl_default"];
    for attr in &acl_attrs {
        let xattr = match file.get_xattr(attr) {
            Ok(item) => item,
            Err(_) => None,
        };
        if let Some(ref data) = xattr {
            if !data.is_empty() {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn list_args(meta: &std::fs::Metadata, path: &std::path::Path) -> String {
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
    let has_acl = match has_acl(path){
        Ok(true) => true,
        _ => false,
    };
    if has_acl {
        perms.push('+');
    }
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
fn handle_show_entries(args: &Cmd, nlink_w: usize, owner_w: usize, group_w: usize, size_w: usize) {
    let entries = [".", ".."];
    if args.flags.contains(&"l".to_string()) {
        for entry in &entries {
            if let Ok(meta) = std::fs::metadata(entry) {
                let perms = list_args(&meta, Path::new(entry));
                let nlinks = meta.nlink();
                let size = meta.len();
                let date = get_time_meta(&meta);
                let (username, group) = get_group_and_user_meta(&meta);
                write_(&format!(
                    "{} {:>width_n$} {:<width_o$} {:<width_g$} {:>width_s$} {} {}\n",
                    perms, nlinks, username, group, size, date, entry,
                    width_n = nlink_w, width_o = owner_w, width_g = group_w, width_s = size_w
                ));
            }
        }
    } else {
        write_(".  ..  ");
    }
}

fn major(dev: u64) -> u64 {
    (dev >> 8) & 0xfff
}
fn minor(dev: u64) -> u64 {
    (dev & 0xff) | ((dev >> 12) & 0xfff00)
}

fn get_target(args: &Cmd) -> Vec<String> {
    if args.args.is_empty() {
        vec![".".to_string()]
    } else {
        args.args.clone()
    }
}

fn print_error(target: &str) {
    write_(&format!("ls : cannot access {}: No such file or dir\n", target));
}

fn format_long_listing(perms: &str, nlinks: u64, username: &str, group: &str, size: &str, date: &str, name: &str, nlink_w: usize, owner_w: usize, group_w: usize, size_w: usize) -> String {
    format!(
        "{} {:>width_n$} {:<width_o$} {:<width_g$} {:^width_s$} {} {}\n",
        perms, nlinks, username, group, size, date, name,
        width_n = nlink_w, width_o = owner_w, width_g = group_w, width_s = size_w
    )
}

fn print_file(target: &str, meta: &std::fs::Metadata, args: &Cmd) {
    if args.flags.contains(&"l".to_string()) {
        let perms = list_args(&meta, Path::new(&target));
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
        let mut name = target.to_string();
        if meta.file_type().is_symlink() {
            let target_path = read_link(target).unwrap_or_default();
            name = format!("{} -> {}", target, target_path.to_string_lossy());
        }
        let nlink_w = nlinks.to_string().len().max(1);
        let owner_w = username.len().max(1);
        let group_w = group.len().max(1);
        let size_w = size.len().max(1);
        write_(&format_long_listing(&perms, nlinks, &username, &group, &size, &date, &name, nlink_w, owner_w, group_w, size_w));
    } else {
        write_(&format!("{}\n", target));
    }
}

fn print_entry_long(elems: &DirEntry, args: &Cmd, output: &mut String, nlink_w: usize, owner_w: usize, group_w: usize, size_w: usize) {
    let perms = match elems.metadata() {
        Ok(m) => list_args(&m, &elems.path()),
        Err(_) => String::from("?"),
    };
    let nlinks = match elems.metadata() {
        Ok(meta) => meta.nlink(),
        Err(_) => 0,
    };
    let (username, group) = get_group_and_user(elems);
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
    let date = get_time(elems);
    let show = args.flags.contains(&"a".to_string());
    let mut name = String::new();
    match check_type(elems) {
        Types::Dir(n) => {
            let name_str = n.to_string_lossy();
            let display = if args.flags.contains(&"F".to_string()) {
                format!("{}/", name_str)
            } else {
                name_str.to_string()
            };
            name = format!("\x1b[34m{}\x1b[0m", display);
        }
        Types::Executable(n) => {
            let name_str = n.to_string_lossy();
            name = format!("\x1b[32m{}\x1b[0m", name_str);
        }
        Types::Symlink(_n) => {
            let file_name_os = elems.file_name();
            let file_name = file_name_os.to_string_lossy();
            let target = read_link(elems.path()).unwrap_or_default().to_string_lossy().to_string();
            name = format!("{} -> {}", file_name, target);
        }
        Types::File(n)
        | Types::CharDevice(n)
        | Types::BlockDevice(n)
        | Types::Socket(n)
        | Types::Pipe(n) => {
            let name_str = n.to_string_lossy();
            name = name_str.to_string();
        }
        _ => (),
    }
    let name_stripped = strip_ansi_escapes(&name);
    if show || !name_stripped.starts_with('.') {
        write_(&format_long_listing(&perms, nlinks, &username, &group, &size_str, &date, &name, nlink_w, owner_w, group_w, size_w));
    }
    output.clear();
}

fn print_entry_short(elems: &DirEntry, args: &Cmd, output: &mut String) {
    let show = args.flags.contains(&"a".to_string());
    match check_type(elems) {
        Types::Dir(name) => {
            let name_str = name.to_string_lossy();
            let display = if args.flags.contains(&"F".to_string()) {
                format!("{}/", name_str)
            } else {
                name_str.to_string()
            };
            let colored = format!("\x1b[34m{}\x1b[0m", display);
            if show {
                output.push_str(&colored);
                output.push_str("  ");
            } else if !name_str.starts_with('.') {
                output.push_str(&colored);
                output.push_str("  ");
            }
        }
        Types::Executable(name) => {
            let name_str = name.to_string_lossy();
            if args.flags.contains(&"F".to_string()) {
                if show {
                    output.push_str(&format!("\x1b[32m{}\x1b[0m*  ", name_str));
                } else if !name_str.starts_with('.') {
                    output.push_str(&format!("\x1b[32m{}\x1b[0m*  ", name_str));
                }
            } else {
                if show {
                    output.push_str(&format!("\x1b[32m{}\x1b[0m  ", name_str));
                } else if !name_str.starts_with('.') {
                    output.push_str(&format!("\x1b[32m{}\x1b[0m  ", name_str));
                }
            }
        }
        Types::Symlink(_name) => {
            let file_name_os = elems.file_name();
            let file_name = file_name_os.to_string_lossy();
            if args.flags.contains(&"F".to_string()) {
                let display = format!("{}@  ", file_name);
                if show {
                    output.push_str(&display);
                } else if !file_name.starts_with('.') {
                    output.push_str(&display);
                }
            } else {
                let display = format!("{}  ", file_name);
                if show {
                    output.push_str(&display);
                } else if !file_name.starts_with('.') {
                    output.push_str(&display);
                }
            }
        }
        Types::File(name)
        | Types::CharDevice(name)
        | Types::BlockDevice(name)
        | Types::Socket(name)
        | Types::Pipe(name) => {
            let name_str = name.to_string_lossy();
            if show {
                output.push_str(&format!("{}  ", name_str));
            } else if !name_str.starts_with('.') {
                output.push_str(&format!("{}  ", name_str));
            }
        }
        _ => (),
    }
}

fn print_directory(target: &str, args: &Cmd) {
    let readir = fs::read_dir(&target);
    let mut entries: Vec<_> = match readir {
        Ok(rd) => rd.filter_map(Result::ok).collect(),
        Err(_) => {
            print_error(target);
            return;
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

        let mut nlink_w = 1;
        let mut owner_w = 1;
        let mut group_w = 1;
        let mut size_w = 1;
        for entry in &entries {
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let nlinks = meta.nlink().to_string().len();
            let (username, group) = get_group_and_user(entry);
            let size = meta.len().to_string().len();
            nlink_w = nlink_w.max(nlinks);
            owner_w = owner_w.max(username.len());
            group_w = group_w.max(group.len());
            size_w = size_w.max(size);
        }
        // Also check target and target/.. for width calculation if show_all
        if show_all {
            for special in [&target[..], &format!("{}/..", &target)[..]] {
                if let Ok(meta) = metadata(special) {
                    let nlinks = meta.nlink().to_string().len();
                    let (username, group) = get_group_and_user_meta(&meta);
                    let size = meta.len().to_string().len();
                    nlink_w = nlink_w.max(nlinks);
                    owner_w = owner_w.max(username.len());
                    group_w = group_w.max(group.len());
                    size_w = size_w.max(size);
                }
            }
        }
        let show = args.flags.contains(&"a".to_string());
        if show {
            handle_show_entries(args, nlink_w, owner_w, group_w, size_w);
        }
        let mut output = String::new();
        for elems in entries {
            if args.flags.contains(&"l".to_string()) {
                print_entry_long(&elems, args, &mut output, nlink_w, owner_w, group_w, size_w);
            } else {
                print_entry_short(&elems, args, &mut output);
            }
            if !output.is_empty() && !args.flags.contains(&"l".to_string()) {
                write_(&format!("{}\n", output));
                output.clear();
            }
        }
    } else {
        let show = args.flags.contains(&"a".to_string());
        if show {
            handle_show_entries(args, 1, 1, 1, 1);
        }
        let mut output = String::new();
        for elems in entries {
            print_entry_short(&elems, args, &mut output);
        }
        if !output.is_empty() {
            write_(&format!("{}\n", output.trim_end()));
        }
    }
}

pub fn ls(_shell: &mut Shell, args: &Cmd) {
    let targets = get_target(args);
    for target in targets {
        let meta = std::fs::metadata(&target);
        match meta {
            Ok(meta) => {
                if meta.is_dir() {
                    print_directory(&target, args);
                } else {
                    print_file(&target, &meta, args);
                }
            }
            Err(_) => {
                print_error(&target);
            }
        }
    }
}

fn strip_ansi_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            while let Some(nc) = chars.next() {
                if nc == 'm' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

