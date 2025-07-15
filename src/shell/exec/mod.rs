pub mod builtins;

use crate::shell::Shell;
use crate::shell::parse::Cmd;

use std::collections::HashMap;
use std::ffi::OsString;

pub use builtins::{
    base::{clear, echo, exit, help, pwd, touch},
    cat, cd, copy, list, mkdir, mv, remove, write_,
};

unsafe extern "C" {
    pub fn fork() -> i32;
    pub fn wait(status: *mut i32) -> i32;
}

pub fn execution(shell: &mut Shell, command: Cmd) {
    unsafe {
        match shell.builtins.get(&command.exec) {
            Some(func) => match command.exec.as_str() {
                "cd" | "exit" => func(shell, &command),
                _ => {
                    let pid = fork();
                    if pid < 0 {
                        write_("Fork failed\n");
                        return;
                    } else if pid == 0 {
                        func(shell, &command);
                        // println!("Child process {} finished", getppid());
                        std::process::exit(0);
                    } else {
                        let mut status = 0;
                        wait(&mut status);
                        if status != 0 {
                            write_("error wait\n");
                        } else {
                            // println!("child {} finished", pid);
                        }
                    }
                }
            },
            None => {
                write_(&format!("Command not found: {}\n", command.exec));
            }
        }
    }
}

pub fn get_builtins() -> HashMap<String, fn(&mut Shell, &Cmd)> {
    HashMap::from([
        ("help".to_string(), help as fn(&mut Shell, &Cmd)),
        ("exit".to_string(), exit as fn(&mut Shell, &Cmd)),
        ("clear".to_string(), clear as fn(&mut Shell, &Cmd)),
        ("echo".to_string(), echo as fn(&mut Shell, &Cmd)),
        ("pwd".to_string(), pwd as fn(&mut Shell, &Cmd)),
        ("touch".to_string(), touch as fn(&mut Shell, &Cmd)),
        ("ls".to_string(), list::ls as fn(&mut Shell, &Cmd)),
        ("cd".to_string(), cd::cd as fn(&mut Shell, &Cmd)),
        ("mv".to_string(), mv::mv as fn(&mut Shell, &Cmd)),
        ("cat".to_string(), cat::cat as fn(&mut Shell, &Cmd)),
        ("mkdir".to_string(), mkdir::mkdir as fn(&mut Shell, &Cmd)),
        ("rm".to_string(), remove::rm as fn(&mut Shell, &Cmd)),
        ("cp".to_string(), copy::cp as fn(&mut Shell, &Cmd)),
    ])
}
