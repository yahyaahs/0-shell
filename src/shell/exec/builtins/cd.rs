use crate::shell::Shell;
use std::{env, fs, path::PathBuf};

pub fn cd(shell: &mut Shell, args: &Vec<String>) {
    if args.len() == 0 {
        let home = env::home_dir();
        match home {
            Some(path) => shell.cwd = path,
            None => println!("cd: no such file or directory: home"),
        }
    } else if args.len() == 1 {
        let new_path = &args[0];
        if new_path.starts_with('/') || new_path.starts_with('~') {
            let data = fs::read_dir(new_path);
            match data {
                Ok(_) => shell.cwd = PathBuf::from(new_path),
                Err(_) => println!("cd: no such file or directory: {}", new_path),
            };
        } else {
            let relative_new_path = shell.cwd.join(new_path);
            let data = fs::read_dir(&relative_new_path);
            match data {
                Ok(_) => shell.cwd = PathBuf::from(relative_new_path),
                Err(_) => println!("cd: no such file or directory: {}", new_path),
            };
        }
    } else {
        println!("cd: args not suported: {}", args.join(" "));
    }

    shell.update_prompt();
}
