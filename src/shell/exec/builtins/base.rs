use std::{collections::HashMap, fs::OpenOptions, io::ErrorKind, path::PathBuf};

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

pub fn help(_shell: &mut Shell, cmd: &Cmd) {
    let help_texts = get_help_texts();

    match cmd.args.len() {
        0 => {
            println!("Usage: help [command]");
            println!("Supported commands:");
            for command in help_texts.keys() {
                println!("\t{}", command);
            }
        }
        1 => {
            let command = &cmd.args[0];
            match help_texts.get(command.as_str()) {
                Some(text) => println!("{}", text),
                None => println!("help: no help topics match '{}'", command),
            }
        }
        _ => {
            println!("help: too many arguments");
            println!("Usage: help [command]");
        }
    }
}

fn get_help_texts() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();

    map.insert("exit", "exit: Exit the shell..\n\tUsage: exit [status]");
    map.insert(
        "echo",
        "echo [args...]: Print arguments to the standard output.\n\tUsage: echo \"helloword\"",
    );
    map.insert("pwd", "pwd: Print the current working directory.\n\tUsage: pwd");
    map.insert(
        "touch",
        "touch [file...]: Create empty file(s).\n\tUsage: touch [file]",
    );
    map.insert("ls", "ls: List directory contents.\n\tUsage: ls -[lfa] [dir]");
    map.insert("cd", "cd [dir]: Change the current directory.\n\tUsage: cd [dir]");
    map.insert("cat", "cat [file...]: Concatenate and display file(s).\n\tUsage: cat [file]");
    map.insert("mkdir", "mkdir [dir...]: Create new directories.\n\tUsage: mkdir [dir]");
    map.insert(
        "rm",
        "rm [file...]: Remove file(s) or directory recursively if implemented.\n\tUsage: rm -[r] [file]",
    );
    map.insert("cp", "cp [src] [dest]: Copy file from src to dest.\n\tUsage: pc [src] [dest]");

    map
}
