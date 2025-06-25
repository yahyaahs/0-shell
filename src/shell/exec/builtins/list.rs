use super::helper::{Types, check_type};
use crate::shell::Shell;
use crate::shell::parse::Cmd;

use chrono::DateTime;
use std::fs::{DirEntry, File};
use std::io::{BufRead, BufReader};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use core::arch::asm;

pub fn ls(_shell: &mut Shell, cmd: &Cmd) {
    let paths = std::fs::read_dir(".").unwrap();
    let mut output = String::new();
    let show = cmd.flags.contains(&"a".to_string());
    let classify = cmd.flags.contains(&"F".to_string());
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
        if cmd.flags.contains(&"l".to_string()) {
            unsafe {
                (perms, (username, group), size, date) = read_file_data(&mut elems);
            }
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
        if cmd.flags.contains(&"l".to_string()) && !output.is_empty() {
            println!(
                "{}",
                format!(
                    "{} {} {} {:>5} {:>5} {}",
                    perms, username, group, size, date, output
                )
            );
            output.clear();
        } else if !output.is_empty() {
            println!("{}", output);
            output.clear();
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
struct Stat {
    st_dev: u64,
    st_ino: u64,
    st_nlink: u64,
    st_mode: u32,
    st_uid: u32,
    st_gid: u32,
    st_rdev: u64,
    st_size: i64,
    st_blksize: i64,
    st_blocks: i64,
    st_atime: u64,      // st_atim.tv_sec
    st_atime_nsec: u64, // st_atim.tv_nsec
    st_mtime: u64,      // st_mtim.tv_sec
    st_mtime_nsec: u64, // st_mtim.tv_nsec
    st_ctime: u64,      // st_ctim.tv_sec
    st_ctime_nsec: u64, // st_ctim.tv_nsec
}

const AT_FDCWD: i32 = -100;

unsafe fn read_file_data(entry: &mut DirEntry) -> (String, (String, String), u64, String) {
    let mut path_bytes = entry.path().as_os_str().as_bytes().to_vec(); // Convert to &[u8]
    path_bytes.push(0); // Null-terminate '\0'

    let path = path_bytes.as_ptr(); // Get raw pointer

    let stat_buf = Stat {
        st_dev: 0,
        st_ino: 0,
        st_nlink: 0,
        st_mode: 0,
        st_uid: 0,
        st_gid: 0,
        st_rdev: 0,
        st_size: 0,
        st_blksize: 0,
        st_blocks: 0,
        st_atime: 0,
        st_atime_nsec: 0,
        st_mtime: 0,
        st_mtime_nsec: 0,
        st_ctime: 0,
        st_ctime_nsec: 0,
    };

    let ret: isize;

    unsafe {
        asm!(
            "syscall",
            in("rax") 262,            // syscall number for newfstatat
            in("rdi") AT_FDCWD,       // dirfd = current directory
            in("rsi") path,           // path pointer
            in("rdx") &stat_buf,       // stat buf pointer
            in("r10") 0,              // flags
            lateout("rax") ret,
        );
    }

    if ret < 0 {
        println!("err syscall fail");
    }
    // println!("{ret}");

    // Print in rwx format:
    fn to_rwx(bits: u8) -> &'static str {
        match bits {
            0b111 => "rwx",
            0b110 => "rw-",
            0b101 => "r-x",
            0b100 => "r--",
            0b011 => "-wx",
            0b010 => "-w-",
            0b001 => "--x",
            0b000 => "---",
            _ => "???",
        }
    }

    let mode = stat_buf.st_mode;
    let perms = mode & 0o777; // lower 9 bits = rwxrwxrwx

    let owner = (perms & 0o700) >> 6;
    let group = (perms & 0o070) >> 3;
    let other = perms & 0o007;

    let permission = format!(
        "{}{}{}{}",
        file_type_char(mode),
        to_rwx(owner as u8),
        to_rwx(group as u8),
        to_rwx(other as u8)
    );

    let uid = get_id_to_name(stat_buf.st_uid, "/etc/passwd").unwrap();
    let gid = get_id_to_name(stat_buf.st_gid, "/etc/group").unwrap();
    let size = stat_buf.st_size;
    let time = format_timestamp(
        stat_buf.st_mtime as f64 + (stat_buf.st_mtime_nsec as f64) / 1_000_000_000.0,
    );

    (permission, (uid, gid), size as u64, time)
}

fn get_id_to_name(uid: u32, source: &str) -> Option<String> {
    let file = File::open(source).ok()?;
    let reader = BufReader::new(file);
    let id_str = uid.to_string();

    for line_res in reader.lines() {
        if let Ok(line) = line_res {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 2 && parts[2] == id_str {
                return Some(parts[0].to_string());
            }
        }
    }
    None
}

fn format_timestamp(timestamp: f64) -> String {
    let secs = timestamp.trunc() as i64;
    let nsecs = ((timestamp.fract()) * 1_000_000_000.0) as u32;
    let naive = DateTime::from_timestamp(secs, nsecs).expect("Invalid timestamp");
    naive.format("%d %b %y %H:%M").to_string()
}

fn file_type_char(mode: u32) -> char {
    match mode & 0o170000 {
        0o040000 => 'd', // directory
        0o100000 => '-', // regular file
        0o120000 => 'l', // symlink
        0o010000 => 'p', // FIFO (pipe)
        0o020000 => 'c', // character device
        0o060000 => 'b', // block device
        0o140000 => 's', // socket
        _ => '?',        // unknown
    }
}

// struct stat {
//     dev_t     st_dev;     /* Device ID */
//     ino_t     st_ino;     /* Inode number */
//     nlink_t   st_nlink;   /* Number of hard links */
//     mode_t    st_mode;    /* File mode (type and permissions) */
//     uid_t     st_uid;     /* Owner user ID */
//     gid_t     st_gid;     /* Owner group ID */
//     dev_t     st_rdev;    /* Device ID (if special file) */
//     off_t     st_size;    /* Total size, in bytes */
//     blksize_t st_blksize; /* Block size for filesystem I/O */
//     blkcnt_t  st_blocks;  /* Number of blocks allocated */
//     struct timespec st_atim; /* Time of last access */
//     struct timespec st_mtim; /* Time of last modification */
//     struct timespec st_ctim; /* Time of last status change */
// };
