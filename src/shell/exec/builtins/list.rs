use super::*;
use chrono::{DateTime, Local};
use std::fs::File;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use std::{
    fs::{self, DirEntry, metadata, read_link},
    os::unix::fs::{MetadataExt, PermissionsExt},
    time::{Duration, SystemTime},
};
use users::{get_group_by_gid, get_user_by_uid};
use xattr::FileExt;
use terminal_size::{Width, terminal_size};

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
    let file = match File::open(path) {
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
    let has_acl = match has_acl(path) {
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
    let under_six = Duration::from_secs(60 * 60 * 24 * 30 * 6); // 6 months
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

fn handle_show_entries(
    args: &Cmd,
    nlink_w: usize,
    owner_w: usize,
    group_w: usize,
    size_w: usize,
    output: &mut Vec<String>,
) {
    let entries = [".", ".."];
    if args.flags.contains(&"l".to_string()) {
        for entry in &entries {
            if let Ok(meta) = metadata(entry) {
                if args.flags.contains(&"F".to_string()) {
                    output.push(format!(
                        "{} {:>width_n$} {:<width_o$} {:<width_g$} {:>width_s$} {} {}/\n",
                        list_args(&meta, Path::new(entry)),
                        meta.nlink(),
                        get_group_and_user_meta(&meta).0,
                        get_group_and_user_meta(&meta).1,
                        meta.len(),
                        get_time_meta(&meta),
                        entry,
                        width_n = nlink_w,
                        width_o = owner_w,
                        width_g = group_w,
                        width_s = size_w
                    ));
                } else {
                    output.push(format!(
                        "{} {:>width_n$} {:<width_o$} {:<width_g$} {:>width_s$} {} {}\n",
                        list_args(&meta, Path::new(entry)),
                        meta.nlink(),
                        get_group_and_user_meta(&meta).0,
                        get_group_and_user_meta(&meta).1,
                        meta.len(),
                        get_time_meta(&meta),
                        entry,
                        width_n = nlink_w,
                        width_o = owner_w,
                        width_g = group_w,
                        width_s = size_w
                    ));
                }
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

fn print_error(target: &str, err: &std::io::Error) {
    write_(&format!(
        "ls: cannot access '{}': {}\n",
        target,
        err
    ));
}

fn print_file(target: &str, meta: &std::fs::Metadata, args: &Cmd) {
    if args.flags.contains(&"l".to_string()) {
        let size = {
            let file_type = meta.file_type();
            if file_type.is_char_device() || file_type.is_block_device() {
                let rdev = meta.rdev();
                format!("{:>3}, {:>3}", major(rdev), minor(rdev))
            } else {
                format!("{}", meta.len())
            }
        };
        let mut name = target.to_string();
        if meta.file_type().is_symlink() {
            match read_link(target) {
                Ok(target_path) => {
                    name = format!("{} -> {}", target, target_path.to_string_lossy());
                }
                Err(err) => {
                    write_(&format!(
                        "ls: cannot read symbolic link '{}': {}\n",
                        target,
                        err
                    ));
                }
            }
        }
        let nlink_w = meta.nlink().to_string().len().max(2);
        let (username, group) = get_group_and_user_meta(&meta);
        let owner_w = username.len().max(2);
        let group_w = group.len().max(2);
        let size_w = size.len().max(4);
        let perms = list_args(&meta, Path::new(&target));
        let date = get_time_meta(&meta);
        write_(&format!(
            "{:<10} {:>nlink_w$} {:<owner_w$} {:<group_w$} {:>size_w$} {} {}\n",
            perms,
            meta.nlink(),
            username,
            group,
            size,
            date,
            name,
            nlink_w = nlink_w,
            owner_w = owner_w,
            group_w = group_w,
            size_w = size_w
        ));
    } else {
        write_(&format!("{}\n", target));
    }
}

fn print_entry_long(
    elems: &DirEntry,
    args: &Cmd,
    nlink_w: usize,
    owner_w: usize,
    group_w: usize,
    size_w: usize,
    output: &mut Vec<String>,
) {
    let show_all = args.flags.contains(&"a".to_string());
    let flag_f = args.flags.contains(&"F".to_string());
    let meta = match elems.metadata() {
        Ok(m) => m,
        Err(err) => {
            output.push(format!(
                "ls: cannot access '{}': {}\n",
                elems.path().to_string_lossy(),
                err
            ));
            return;
        }
    };
    let perms = list_args(&meta, &elems.path());
    let nlinks = meta.nlink();
    let (username, group) = get_group_and_user(elems);
    let file_type = meta.file_type();
    let size_str = {
        if file_type.is_char_device() || file_type.is_block_device() {
            let rdev = meta.rdev();
            format!("{:>3}, {:>3}", major(rdev), minor(rdev))
        } else {
            format!("{}", meta.len())
        }
    };
    let date = get_time_meta(&meta);
    let entry_type = check_type(elems);
    let name = match format_entry_name(elems, &entry_type, flag_f, args.flags.contains(&"l".to_string())) {
        Ok(n) => n,
        Err(err_msg) => {
            output.push(err_msg);
            return;
        }
    };
    if show_all || !name.starts_with('.') {
        output.push(format!(
            "{:<10} {:>nlink_w$} {:<owner_w$} {:<group_w$} {:>size_w$} {} {}\n",
            perms,
            nlinks,
            username,
            group,
            size_str,
            date,
            name,
            nlink_w = nlink_w,
            owner_w = owner_w,
            group_w = group_w,
            size_w = size_w
        ));
    }
}

fn format_entry_name(
    entry: &DirEntry,
    entry_type: &Types,
    flag_f: bool,
    flag_l : bool,
) -> Result<String, String> {
    match entry_type {
        Types::Dir(n) => {
            let name_str = n.to_string_lossy();
            if flag_f {
                Ok(format!("{}/", name_str))
            } else {
                Ok(name_str.to_string())
            }
        }
        Types::Executable(n) => Ok(n.to_string_lossy().to_string()),
        Types::Symlink(_n) => {
            let file_name_os = entry.file_name();
            let file_name = file_name_os.to_string_lossy();
            match read_link(entry.path()) {
                Ok(link) => {
                    if flag_f {
                        if flag_l {
                            Ok(format!("{} -> {}", file_name, link.to_string_lossy()))
                        } else {
                            Ok(format!("{}@", file_name))
                        }
                    } else {
                        if flag_l {
                            Ok(format!("{} -> {}", file_name, link.to_string_lossy()))
                        } else {
                            Ok(file_name.to_string())
                        }
                    }
                },
                Err(err) => Err(format!(
                    "ls: cannot read symbolic link '{}': {}\n",
                    entry.path().to_string_lossy(),
                    err
                )),
            }
        }
        Types::File(n) | Types::CharDevice(n) | Types::BlockDevice(n) => {
            Ok(n.to_string_lossy().to_string())
        }
        Types::Socket(name) => Ok(format!("{}=", name.to_string_lossy())),
        Types::Pipe(name) => Ok(format!("{}|", name.to_string_lossy())),
        _ => Ok(String::new()),
    }
}

fn get_terminal_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    }
}

fn print_in_columns(entries: &[String], term_width: usize) {
    if entries.is_empty() {
        return;
    }
    let max_len = entries.iter().map(|s| s.len()).max().unwrap();
    let col_width = max_len + 2;
    let cols = (term_width / col_width).max(1);
    let rows = (entries.len() + cols - 1) / cols;

    for row in 0..rows {
        for col in 0..cols {
            let idx = col * rows + row;
            if idx < entries.len() {
                let entry = &entries[idx];
                write_(&format!("{:width$}", entry, width = col_width));
            }
        }
        write_("\n");
    }
}

fn print_directory(target: &str, args: &Cmd) {
    let long_listing = args.flags.contains(&"l".to_string());
    let show_all = args.flags.contains(&"a".to_string());
    let readir: Result<fs::ReadDir, io::Error> = fs::read_dir(&target);
    let entries: Vec<_> = match readir {
        Ok(rd) => {
            let mut entr = vec![];
            for el in rd {
                match el {
                    Ok(entry) => {
                        entr.push(entry);
                    }
                    Err(err) => {
                        print_error(target, &err);
                    }
                }
            }
            entr
        }
        Err(err) => {
            print_error(target, &err);
            vec![]
        }
    };
    if long_listing {
        let mut total_blocks = 0;

        for entry in &entries {
            let filename = entry.file_name();
            let name = filename.to_string_lossy();
            if !show_all && name.starts_with('.') {
                continue;
            }

            if let Ok(meta) = entry.metadata() {
                total_blocks += meta.blocks();
            }
        }

        if show_all {
            let current = Path::new(target).join(".");
            if let Ok(meta) = metadata(&current) {
                total_blocks += meta.blocks();
            }

            let parent = Path::new(target).join("..");
            if let Ok(meta) = metadata(&parent) {
                total_blocks += meta.blocks();
            }
        }
        let mut formated_entries = Vec::new();
        formated_entries.push(format!("total: {}\n", total_blocks / 2,));

        let mut nlink_w = 1;
        let mut owner_w = 1;
        let mut group_w = 1;
        let mut size_w = 1;
        let mut perms_w = 1;
        for entry in &entries {
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(err) => {
                    print_error(&entry.path().to_string_lossy(), &err);
                    continue;
                }
            };
            let perms = list_args(&meta, &entry.path());
            let nlinks = meta.nlink().to_string().len();
            let (username, group) = get_group_and_user(entry);
            let file_type = meta.file_type();
            let size_len = if file_type.is_char_device() || file_type.is_block_device() {
                let maj = major(meta.rdev());
                let min = minor(meta.rdev());
                format!("{:>3}, {:>3}", maj, min).len()
            } else {
                meta.len().to_string().len()
            };
            nlink_w = nlink_w.max(nlinks);
            owner_w = owner_w.max(username.len());
            group_w = group_w.max(group.len());
            size_w = size_w.max(size_len);
            perms_w = perms_w.max(perms.len());
        }
        if show_all {
            for special in [target, format!("{}/..", target).as_str()] {
                if let Ok(meta) = metadata(special) {
                    let perms = list_args(&meta, Path::new(special));
                    let nlinks = meta.nlink().to_string().len();
                    let (username, group) = get_group_and_user_meta(&meta);
                    let file_type = meta.file_type();
                    let size_len = if file_type.is_char_device() || file_type.is_block_device() {
                        let maj = major(meta.rdev());
                        let min = minor(meta.rdev());
                        format!("{:>3}, {:>3}", maj, min).len()
                    } else {
                        meta.len().to_string().len()
                    };
                    nlink_w = nlink_w.max(nlinks);
                    owner_w = owner_w.max(username.len());
                    group_w = group_w.max(group.len());
                    size_w = size_w.max(size_len);
                    perms_w = perms_w.max(perms.len());
                }
            }
            handle_show_entries(
                args,
                nlink_w,
                owner_w,
                group_w,
                size_w,
                &mut formated_entries,
            );
        }
        for elems in entries {
            print_entry_long(
                &elems,
                args,
                nlink_w,
                owner_w,
                group_w,
                size_w,
                &mut formated_entries,
            )
        }

        formated_entries.sort_by(|a, b| {
            fn entry_priority(entry: &str) -> u8 {
                if entry.contains("Permission denied") {
                    0
                } else if entry.starts_with("total: ") {
                    1
                } else if entry.trim_end().ends_with(" .")  {
                    2
                }else if  entry.trim_end().ends_with(" .."){
                    3
                } else {
                    4
                }
            }

            let pa = entry_priority(a);
            let pb = entry_priority(b);

            match pa.cmp(&pb) {
                std::cmp::Ordering::Equal => a.cmp(b),
                other => other,
            }
        });
        write_(&formated_entries.join(""))
    } else {
        let show_all = args.flags.contains(&"a".to_string());
        let flag_f = args.flags.contains(&"F".to_string());
        let mut display_entries = Vec::new();
        if show_all {
            display_entries.push(".".to_string());
            display_entries.push("..".to_string());
        }
        for elems in &entries {
            let mut name = String::new();
            let show = {
                let entry_type = check_type(elems);
                match format_entry_name(elems, &entry_type, flag_f, args.flags.contains(&"l".to_string())) {
                    Ok(n) => {
                        name = n;
                        show_all || !name.starts_with('.')
                    }
                    Err(_) => false,
                }
            };
            if show && !name.is_empty() {
                display_entries.push(name);
            }
        }
        let term_width = get_terminal_width();
        print_in_columns(&display_entries, term_width);
    }
}

pub fn ls(_shell: &mut Shell, args: &Cmd) {
    let targets = get_target(args);
    for (i, target) in targets.iter().enumerate() {
        let meta = metadata(&target);
        match meta {
            Ok(meta) => {
                if meta.is_dir() {
                    if !target.contains(".") && targets.len() > 1 {
                        write_(&format!("{}:\n", &target));
                    }
                    print_directory(&target, args);
                } else {
                    print_file(&target, &meta, args);
                }
            }
            Err(_) => {
                print_error(&target, &std::io::Error::new(std::io::ErrorKind::NotFound, "No such file or directory"));
            }
        }
        if i != targets.len() - 1 {
            write_("\n");
        }
    }
}
