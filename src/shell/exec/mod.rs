pub mod builtins;
pub mod list;
pub use builtins::*;

use crate::shell::parse::Cmd;

use super::Shell;

pub fn execute_command(shell: &Shell, command: &Cmd) {
    match shell.builtins.get(&command.exec) {
        Some(func) => func(&command.args),
        None => match find_non_builtins(&command.exec) {
            Some(_) => {
                println!("{}: command not found", command.exec.trim());
                // let output = Command::new(cmd.clone())
                // .args(&command.args)
                // .output()
                // .expect("failed to execute process");
                // if output.status.success() {
                // let stdout = String::from_utf8_lossy(&output.stdout);
                // print!("{}", stdout.replace(&cmd, &command.exec));
                // } else {
                // let stderr = String::from_utf8_lossy(&output.stderr);
                // print!("{}", stderr.replace(&cmd, &command.exec));
                // }
            }
            None => println!("{}: command not found", command.exec.trim()),
        },
    };
}
