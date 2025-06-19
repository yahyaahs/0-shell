use crate::shell::{Shell, parse::Cmd, update_prompt};
use std::{env, path::PathBuf};

pub fn cd(shell: &mut Shell, cmd: &Cmd) {
    if cmd.args.len() == 0 {
        let home = env::home_dir();
        match home {
            Some(path) => shell.cwd = path,
            None => println!("cd: no such file or directory: home"),
        }
    } else if cmd.args.len() == 1 {
        let home_dir = match env::home_dir() {
            Some(rv_path) => cmd.args[0].replace("~", rv_path.to_str().unwrap()),
            None => "~".to_string(),
        };
        let new_path = if cmd.args[0].starts_with('~') {
            match env::home_dir() {
                Some(rv_path) => cmd.args[0].replace("~", rv_path.to_str().unwrap()),
                None => {
                    println!("cd: no such file or directory: {}", cmd.args[0]);
                    return;
                }
            }
        } else {
            cmd.args[0].clone()
        };

        if new_path.starts_with('/') {
            let data = env::set_current_dir(new_path.clone());
            match data {
                Ok(_) => shell.cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
                Err(_) => println!(
                    "cd: no such file or directory: {}",
                    new_path.replace(&home_dir, "~")
                ),
            };
        } else {
            let relative_new_path = shell.cwd.join(new_path.clone());
            let data = env::set_current_dir(&relative_new_path);
            match data {
                Ok(_) => shell.cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
                Err(_) => println!(
                    "cd: no such file or directory: {}",
                    new_path.replace(&home_dir, "~")
                ),
            };
        }
    } else {
        println!("cd: args not suported: {}", cmd.args.join(" "));
    }

    update_prompt(shell);
}
