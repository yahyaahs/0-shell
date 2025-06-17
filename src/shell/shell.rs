use super::Shell;
use super::exec::*;
use super::parse::*;

use std::env;
use std::io::Write;
use std::io::{stdin, stdout};
use std::path::PathBuf;

use crate::shell::State;
use crate::shell::exec::builtins::get_builtins;

impl Shell {
    pub fn new() -> Shell {
        Shell {
            pid: std::process::id(),
            cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            env: env::vars().collect(),
            history: Vec::new(),
            last_status: 0,

            builtins: get_builtins(),
            state: State::Ready,
        }
    }

    pub fn run(mut self) {
        let stdin = stdin();
        let mut input = String::new();

        loop {
            match &self.state {
                State::Exec => continue,
                State::Ready => {
                    print!("$ ");
                    stdout().flush().unwrap();
                    input = String::new();
                }
                State::Quote(typ) => {
                    print!("{}> ", typ);
                    self.state = State::Ready;
                    stdout().flush().unwrap();
                }
                State::BackNewLine => {
                    print!("> ");
                    self.state = State::Ready;
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
                        println!("to exec: [{}] [{:?}]", cmd.exec, cmd.args);
                        match state {
                            State::Exec => {
                                self.state = State::Exec;
                                execute_command(&mut self, &cmd);
                            }
                            _ => self.state = state,
                        }
                    }
                    Err(err) => print!("{err}"),
                },
                _ => self.state = state,
            };
        }
    }
}
