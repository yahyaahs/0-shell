pub mod builtins;
pub mod list;

pub use builtins::*;

use crate::shell::parse::Cmd;

use super::{Shell, State};

pub fn execute_command(shell: &mut Shell, command: &Cmd) {
    match shell.builtins.get(&command.exec) {
        Some(func) => func(shell, &command.args),
        None => match find_non_builtins(&command.exec) {
            Some(exec) => println!("{}: command not found [{}]", command.exec.trim(), exec),
            None => println!("{}: command not found", command.exec.trim()),
        },
    };

    shell.state = State::Ready;
}
