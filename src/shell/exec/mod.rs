pub mod builtins;
pub mod helper;

use super::{Shell, State};
use crate::shell::parse::Cmd;
use helper::find_non_builtins;

pub fn execute_command(shell: &mut Shell, command: &Cmd) {
    match shell.builtins.get(&command.exec) {
        Some(func) => func(shell, &command),
        None => match find_non_builtins(&command.exec) {
            Some(exec) => println!("{}: command not found [{}]", command.exec.trim(), exec),
            None => println!("{}: command not found", command.exec.trim()),
        },
    };

    shell.state = State::Ready;
}
