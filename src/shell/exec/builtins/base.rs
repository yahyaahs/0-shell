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
            write_(&format!("exit {}: undefined code\n", cmd.args[0]));
            return;
        }
    };
}

pub fn echo(_shell: &mut Shell, cmd: &Cmd) {
    write_(&format!("{}\n", cmd.args.join(" ")));
}

pub fn pwd(shell: &mut Shell, _cmd: &Cmd) {
    match shell.cwd.to_str() {
        Some(path_str) => write_(&format!("{}\n", path_str)),
        None => write_("Error: Cannot convert current directory to string"),
    }
}

pub fn touch(_shell: &mut Shell, cmd: &Cmd) {
    if cmd.args.is_empty() {
        write_("touch: missing file operand\n");
        return;
    }

    for file in &cmd.args {
        let path = PathBuf::from(file);

        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(_) => {}
            Err(err) => match err.kind() {
                ErrorKind::PermissionDenied => {
                    write_(&format!("touch: {}: Permission denied\n", file))
                }
                ErrorKind::NotFound => write_(&format!(
                    "touch: cannot touch {}: No such file or directory\n",
                    file
                )),
                ErrorKind::IsADirectory => write_(&format!("touch: {}: Is a directory\n", file)),
                _ => write_(&format!("touch: {}: {}\n", file, err)),
            },
        }
    }
}

pub fn help(_shell: &mut Shell, cmd: &Cmd) {
    let help_texts = get_help_texts();

    match cmd.args.len() {
        0 => {
            write_("Usage: help [command]\n");
            write_("Supported commands:\n");
            for command in help_texts.keys() {
                write_(&format!("\t{}\n", command));
            }
        }
        1 => {
            let command = &cmd.args[0];
            match help_texts.get(command.as_str()) {
                Some(text) => write_(&format!("{}\n", text)),
                None => write_(&format!("help: no help topics match '{}'\n", command)),
            }
        }
        _ => {
            write_("help: too many arguments\n");
            write_("Usage: help [command]\n");
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
    map.insert(
        "pwd",
        "pwd: Print the current working directory.\n\tUsage: pwd",
    );
    map.insert(
        "touch",
        "touch [file...]: Create empty file(s).\n\tUsage: touch [file]",
    );
    map.insert(
        "ls",
        "ls: List directory contents.\n\tUsage: ls -[lfa] [dir]",
    );
    map.insert(
        "cd",
        "cd [dir]: Change the current directory.\n\tUsage: cd [dir]",
    );
    map.insert(
        "cat",
        "cat [file...]: Concatenate and display file(s).\n\tUsage: cat [file]",
    );
    map.insert(
        "mkdir",
        "mkdir [dir...]: Create new directories.\n\tUsage: mkdir [dir]",
    );
    map.insert(
        "rm",
        "rm [file...]: Remove file(s) or directory recursively if implemented.\n\tUsage: rm -[r] [file]",
    );
    map.insert(
        "cp",
        "cp [src] [dest]: Copy file from src to dest.\n\tUsage: pc [src] [dest]",
    );

    map
}
