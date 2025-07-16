use super::*;

use std::{
    fs,
    io::{self, ErrorKind},
};

pub fn cat(_shell: &mut Shell, cmd: &Cmd) {
    if cmd.args.len() == 0 {
        infinit_read();
    } else {
        for file in cmd.args.clone() {
            if file == "-".to_string() {
                infinit_read();
            } else {
                let content = fs::read(file.clone());
                match content {
                    Ok(data) => {
                        write_(&data.to_vec().iter().map(|c| *c as char).collect::<String>())
                    }
                    Err(err) => match err.kind() {
                        ErrorKind::PermissionDenied => {
                            write_(&format!("cat: {}: Permission denied\n", file))
                        }
                        ErrorKind::NotFound => {
                            write_(&format!("cat: {}: No such file or directory\n", file))
                        }
                        ErrorKind::IsADirectory => {
                            write_(&format!("cat: {}: Is a directory\n", file))
                        }
                        _ => write_("cat: undefined error\n"),
                    },
                };
            }
        }
    }
}

fn infinit_read() {
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
            break;
        }
        write_(&input);
    }
}
