mod shell;
use std::io::{Write, stdin, stdout};
use std::ops::ShlAssign;
use std::{env, path::PathBuf};

use shell::*;

use crate::shell::exec::{execution, get_builtins};
use crate::shell::parse::{parse_command, scan_command};
unsafe extern "C" {
    fn signal(signal: i32, handler: extern "C" fn(i32));
}
extern "C" fn signal_handler(_signal: i32) {
    print!("\n$");
    stdout().flush().unwrap();

}
fn main() {
    let mut shell = Shell {
        cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
        prompt: "$ ".to_string(),
        builtins: get_builtins(),
        state: shell::State::Ready,
    };

    let stdin = stdin();
    let mut input = String::new();
        unsafe {
        signal(2, signal_handler);
    }

    loop {
        println!("shell state {:#?}", shell.state);
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
                    execution(&mut shell, cmd);
                }
                Err(err) => print!("{err}"),
            },
        };
    }
}
