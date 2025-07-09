mod shell;
use std::io::{Write, stdin, stdout};
use std::sync::{Arc, Mutex};
use std::{env, path::PathBuf};

use shell::*;

use crate::shell::exec::{execute_command, get_builtins};
use crate::shell::parse::{parse_command, scan_command};

fn main() {
    let mut shell = Shell {
        cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
        prompt: "$ ".to_string(),
        builtins: get_builtins(),
        state: shell::State::Ready,
    };

    let stdin = stdin();
    let mut input = String::new();

    loop {
        match &shell.state {
            State::Ready => {
                shell.prompt = String::from(shell.cwd.to_str().unwrap());
                print!("\x1b[0;36;1;96m{} \x1b[0;32;1;96m>\x1b[0m ", shell.prompt);
                stdout().flush().unwrap();
                input = String::new();
            }
            State::Quote(typ) => {
                print!("{}> ", typ);
                shell.state = State::Ready;
                stdout().flush().unwrap();
            }
            State::BackNewLine => {
                print!("> ");
                shell.state = State::Ready;
                stdout().flush().unwrap();
            }
        };

        if input.len() > 0 {
            let mut temp_buffer = String::new();
            stdin.read_line(&mut temp_buffer).unwrap();
            input = format!("{}{}", input, temp_buffer);
        } else {
            stdin.read_line(&mut input).unwrap();
        }

        let input = input.trim();
        let state = scan_command(&input.trim());
        match state {
            Some(new_state) => shell.state = new_state,
            None => match parse_command(&input) {
                Ok(cmd) => {
                    // println!("to exec: [{}] [{:?}] [{:?}]", cmd.exec, cmd.flags, cmd.args);
                    let arc_shell = Arc::new(Mutex::new(shell.clone()));
                    let executor = execute_command(Arc::clone(&arc_shell), cmd.clone());
                    match executor.join() {
                        Ok(_) => shell.state = State::Ready,
                        Err(_) => println!("Error on executing: {}", cmd.exec),
                    };
                }
                Err(err) => print!("{err}"),
            },
        };
    }
}
