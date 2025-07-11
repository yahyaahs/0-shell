use super::*;

use std::{env, path::PathBuf};

pub fn cd(shell: &mut Shell, cmd: &Cmd) {
    if cmd.args.is_empty() {
        let home = env::var("HOME");
        match home {
            Ok(path) => {
                if let Err(_) = env::set_current_dir(&path) {
                    println!("cd: no such file or directory: {}", path);
                } else {
                    shell.cwd = PathBuf::from(path);
                }
            }
            Err(_) => println!("cd: HOME environment variable not set"),
        }
    } else if cmd.args.len() == 1 {
        let target_path = PathBuf::from(&cmd.args[0]);

        let final_path = if target_path.is_absolute() {
            target_path
        } else {
            shell.cwd.join(target_path)
        };

        match env::set_current_dir(&final_path) {
            Ok(_) => {
                shell.cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
            }
            Err(_) => {
                println!("cd: no such file or directory: {}", cmd.args[0]);
            }
        }
    } else {
        println!("cd: too many arguments: {}", cmd.args.join(" "));
    }
}
