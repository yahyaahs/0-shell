use super::Shell;
use super::exec::*;
use super::parse::*;

use std::env;
use std::io;
use std::io::Write;
use std::path::PathBuf;

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
        }
    }

    pub fn run(&self) {
        loop {
            print!("$ ");
            io::stdout().flush().unwrap();

            let stdin = io::stdin();
            let mut input = String::new();

            stdin.read_line(&mut input).unwrap();

            let to_exec = parse_command(&input);
            execute_command(&self, &to_exec);

            // print!("{} {:?}\n", to_exec.exec, to_exec.args);
        }
    }
}
