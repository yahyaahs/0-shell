use super::*;
use crate::shell::Shell;
use std::path::{ PathBuf};
use std::{fs::{self, DirEntry}, os::unix::fs::{MetadataExt, PermissionsExt}, time::{Duration, SystemTime}
};
use users::{get_group_by_gid, get_user_by_uid};
use chrono::{DateTime, Local};
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
            if meta.is_dir() {
                return Types::Dir(name.file_name());
            } else if meta.permissions().mode() & 0o111 != 0 {
                return Types::Executable(name.file_name());
            } else if meta.is_file() {
                return Types::File(name.file_name());
            } else if meta.is_symlink() {
                return Types::Symlink(name.file_name());
            } else {
                return Types::NotSupported;
            }
        }
        _ => Types::Error,
    }
}

pub fn list_arg(args: &DirEntry) -> String {
    let mode = args.metadata().unwrap().permissions().mode();
    let file_type = match mode & 0o170000 {
        0o040000 => 'd', // directory
        0o100000 => '-', // regular file
        0o120000 => 'l', // symlink
        _ => '?',        // other
    };

    let mut perms = String::new();
    perms.push(file_type);

    // User permissions
    perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    // Group permissions
    perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    // Others permissions
    perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    perms
}

pub fn get_group_and_user(args: &DirEntry) -> (String, String) {
    let uid = args.metadata().unwrap().uid();
    let gid = args.metadata().unwrap().gid();
    let username = get_user_by_uid(uid)
        .unwrap()
        .name()
        .to_string_lossy()
        .to_string();
    let group = match get_group_by_gid(gid) {
        Some(g) => g.name().to_string_lossy().to_string(),
        _ => "None".to_string(),
    };
    (username, group)
}
pub fn get_time(args: &DirEntry) -> String{
      let time = match args.metadata().and_then(|m| m.modified()) {
        Ok(mtime) => mtime,
        Err(_) => return "?".to_string(),
    };
    let now = SystemTime::now();
    let under_six = Duration::from_secs(60*60*24*30);
    let passed = match now.duration_since(time){
        Ok(duration)=> duration< under_six ,
        Err(_)=> true,
    };
    let formated: DateTime<Local> = time.into();

    if passed{
        formated.format("%b %e %H:%M").to_string()
    } else {
        formated.format("%b %e  %Y").to_string()
    }


}
pub fn ls(_shell: &mut Shell, args: &Cmd) {
    let paths = fs::read_dir(".").unwrap();
    let mut output = String::new();
    let show = args.flags.contains(&"a".to_string());
    let classify = args.flags.contains(&"F".to_string());
    let mut perms = String::new();
    let mut username = String::new();
    let mut group = String::new();
    let mut size = 0 as u64;
    let mut date = String::new();
    let blue = "\x1b[34m";
    let green = "\x1b[32m";
    let reset = "\x1b[0m";
    let mut paths_hidden = vec![];
    if show {
        paths_hidden.push(PathBuf::from("."));
        paths_hidden.push(PathBuf::from(".."));
    }

    for data in paths {
        let mut elems = data.unwrap();
        if args.flags.contains(&"l".to_string()) {
            perms = list_arg(&mut elems);

            (username, group) = get_group_and_user(&elems);
            size = elems.metadata().unwrap().len();
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
            Types::File(name) | Types::Symlink(name) => {
                let name_str = name.to_string_lossy();
                if show {
                    output.push_str(&name_str);
                } else if !name_str.starts_with('.') {
                    output.push_str(&name_str);
                }
            }
            _ => {}
        }
        if args.flags.contains(&"l".to_string())&& !output.is_empty(){
            println!("{}", format!("{} {} {} {:>5} {:>5} {}", perms , username, group, size, date, output));
            output.clear();
        }else if !output.is_empty(){
            println!("{}", output);
            output.clear();

        }
        // if args.contains(&"l".to_string()){
        //     for dots in paths_hidden{
                
        //     }
        // }
    }

    // for item in output {
    //     println!("{}", item);
    // }
}
