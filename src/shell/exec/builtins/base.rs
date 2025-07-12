use std::{fs::OpenOptions, io::ErrorKind, path::PathBuf};

use super::*;

pub fn exit(_shell: &mut Shell, cmd: &Cmd) {
    if cmd.args.len() == 0 {
        std::process::exit(0)
    };
    match cmd.args[0].parse::<i32>() {
        Ok(nb) => {
            let code = nb.clamp(0, 255);
            std::process::exit(code)
        }
        Err(_) => {
            println!("exit {}: undefined code", cmd.args[0]);
            return;
        }
    };
}

pub fn echo(_shell: &mut Shell, cmd: &Cmd) {
    println!("{}", cmd.args.join(" "));
}

pub fn pwd(shell: &mut Shell, _cmd: &Cmd) {
    println!(
        "{}",
        shell
            .cwd
            .to_str()
            .unwrap_or("cannot find current directory")
    );
}

pub fn touch(_shell: &mut Shell, cmd: &Cmd) {
    if cmd.args.is_empty() {
        println!("touch: missing file operand");
        return;
    }

    for file in &cmd.args {
        let path = PathBuf::from(file);

        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(_) => {}
            Err(err) => match err.kind() {
                ErrorKind::PermissionDenied => println!("touch: {}: Permission denied", file),
                ErrorKind::NotFound => {
                    println!("touch: cannot touch {}: No such file or directory", file)
                }
                ErrorKind::IsADirectory => println!("touch: {}: Is a directory", file),
                _ => println!("touch: {}: {}", file, err),
            },
        }
    }
}
