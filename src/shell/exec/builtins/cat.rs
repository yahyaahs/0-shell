use super::*;

use std::{
    fs,
    io::{self, ErrorKind},
};

unsafe extern "C" {
    fn signal(signal: i32, handler: extern "C" fn(i32));
}

extern "C" fn signal_handler(_signal: i32) {
    println!();
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
                    println!("cat: Input/output error");
                    break;
                }
            };
            if bytes == 0 {
                println!();
                break;
            }
            print!("{}", input);
        }
    } else {
        for file in cmd.args.clone() {
            let content = fs::read_to_string(file.clone());
            match content {
                Ok(data) => print!("{}", data),
                Err(err) => match err.kind() {
                    ErrorKind::PermissionDenied => println!("cat: {}: Permission denied", file),
                    ErrorKind::NotFound => println!("cat: {}: No such file or directory", file),
                    ErrorKind::IsADirectory => println!("cat: {}: Is a directory", file),
                    _ => println!("cat: undefined error"),
                },
            };
        }
    }
}
