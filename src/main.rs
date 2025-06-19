mod shell;
use std::io::{Write, stdin, stdout};
use std::{env, path::PathBuf};

use shell::*;

use crate::shell::exec::execute_command;
use crate::shell::exec::helper::get_builtins;
use crate::shell::parse::{parse_command, scan_command};

fn main() {
    let mut new_shell = Shell {
        pid: std::process::id(),
        cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
        env: env::vars().collect(),
        history: Vec::new(),
        last_status: 0,

        prompt: "$ ".to_string(),
        builtins: get_builtins(),
        state: shell::State::Ready,
    };

    update_prompt(&mut new_shell);
    let stdin = stdin();
    let mut input = String::new();

    loop {
        match &new_shell.state {
            State::Exec => continue,
            State::Ready => {
                print!("{}", new_shell.prompt);
                stdout().flush().unwrap();
                input = String::new();
            }
            State::Quote(typ) => {
                print!("{}> ", typ);
                new_shell.state = State::Ready;
                stdout().flush().unwrap();
            }
            State::BackNewLine => {
                print!("> ");
                new_shell.state = State::Ready;
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
            State::Exec => match parse_command(&input) {
                Ok((state, cmd)) => {
                    println!("to exec: [{}] [{:?}] [{:?}]", cmd.exec, cmd.flags, cmd.args);
                    match state {
                        State::Exec => {
                            new_shell.state = State::Exec;
                            execute_command(&mut new_shell, &cmd);
                        }
                        _ => new_shell.state = state,
                    }
                }
                Err(err) => print!("{err}"),
            },
            _ => new_shell.state = state,
        };
    }
}
