use super::*;

use std::{
    fs,
    io::{self, ErrorKind},
};

unsafe extern "C" {
    fn signal(signal: i32, handler: extern "C" fn(i32));
}

extern "C" fn signal_handler(_signal: i32) {
    std::process::exit(0);
}

pub fn cat(_shell: &mut Shell, cmd: &Cmd) {
    unsafe {
        signal(2, signal_handler);
    }

    if cmd.args.len() == 0 {
        let stdin = io::stdin();
        loop {
            let mut input = String::new();
            let bytes = match stdin.read_line(&mut input) {
                Ok(byt) => byt,
                Err(_) => {
                    write_("cat: Input/output error\n");
                    break;
                }
            };
            if bytes == 0 {
                write_("\n");
                break;
            }
            write_(&input);
        }
    } else {
        for file in cmd.args.clone() {
            let content = fs::read_to_string(file.clone());
            match content {
                Ok(data) => write_(&data),
                Err(err) => match err.kind() {
                    ErrorKind::PermissionDenied => write_(&format!("cat: {}: Permission denied\n", file)),
                    ErrorKind::NotFound => write_(&format!("cat: {}: No such file or directory\n", file)),
                    ErrorKind::IsADirectory => write_(&format!("cat: {}: Is a directory\n", file)),
                    _ => write_("cat: undefined error\n"),
                },
            };
        }
    }
}
