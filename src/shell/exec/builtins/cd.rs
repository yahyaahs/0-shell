use super::*;

use std::{env, io::ErrorKind, path::PathBuf};

pub fn cd(shell: &mut Shell, cmd: &Cmd) {
    if !shell.cwd.exists() {
        write_(&format!(
            "cd: warning: directory {} was deleted, switching to root \"/\"\n",
            shell.cwd.display()
        ));
        let root = PathBuf::from("/");
        if env::set_current_dir(root.clone()).is_err() {
            eprintln!("cd: undefined error\n");
        };
        shell.cwd = root;
        return;
    }

    if cmd.args.is_empty() {
        let home = env::var("HOME");
        match home {
            Ok(path) => {
                if let Err(_) = env::set_current_dir(&path) {
                    write_(&format!("cd: no such file or directory: {}\n", path));
                } else {
                    shell.cwd = PathBuf::from(path);
                }
            }
            Err(_) => write_("cd: cannot find HOME directory set\n"),
        }
    } else if cmd.args.len() == 1 {
        let mut target_path = PathBuf::from(&cmd.args[0]);

        if cmd.args[0].starts_with("~") {
            let home = env::var("HOME");
            match home {
                Ok(path) => target_path = PathBuf::from(cmd.args[0].replace("~", &path)),
                Err(_) => {
                    write_("cd: cannot find HOME directory set\n");
                    return;
                }
            }
        }

        let final_path = if target_path.is_absolute() {
            target_path
        } else {
            shell.cwd.join(target_path)
        };

        if final_path.is_file() {
            write_(&format!("cd: not a directory: {}\n", cmd.args[0]));
            return;
        }

        match env::set_current_dir(&final_path) {
            Ok(_) => {
                shell.cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
            }
            Err(err) => {
                match err.kind() {
                    ErrorKind::PermissionDenied => {
                        write_(&format!("cd: {}: Permission denied\n", cmd.args[0]))
                    }
                    ErrorKind::NotFound => {
                        write_(&format!("cd: {}: No such file or directory\n", cmd.args[0]))
                    }
                    _ => write_("cd: undefined error\n"),
                };
            }
        }
    } else {
        write_(&format!("cd: too many arguments: {}\n", cmd.args.join(" ")));
    }
}
