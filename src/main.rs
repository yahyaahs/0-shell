mod shell;
use std::io::{Read, Write, stdin, stdout};
use std::sync::{Arc, Mutex};
use std::{env, path::PathBuf};

use shell::*;

use crate::shell::exec::helper::get_builtins;
use crate::shell::exec::{StackData, execute_command, join_all};
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

    let shell = Arc::new(Mutex::new(new_shell.clone()));
    let stack = Mutex::new(StackData { processes: vec![] });

    update_prompt(&mut new_shell);
    let mut input = String::new();

    loop {
        match &new_shell.state {
            State::Exec => {
                join_all(&stack);
                new_shell.state = State::Ready;
                continue;
            }
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
            let temp_buffer = match custom_read_line() {
                Some(line) => line,
                None => break,
            };
            input = format!("{}{}", input, temp_buffer);
        } else {
            input = match custom_read_line() {
                Some(line) => line,
                None => break,
            };
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
                            execute_command(Arc::clone(&shell), &stack, cmd);
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

fn custom_read_line() -> Option<String> {
    let mut stdin = stdin();
    let mut res = String::new();
    let mut buff = [0u8; 1];

    loop {
        let size = stdin.read(&mut buff).unwrap();
        if size == 0 {
            if res.len() == 0 {
                println!("\nEOF reached: exiting...");
                return None; // EOF
            } else {
                continue;
            }
        }

        if buff[0] == b'\n' {
            break;
        }

        res.push(buff[0] as char);
    }

    Some(res)
}
