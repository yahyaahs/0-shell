pub mod builtins;

use crate::shell::Shell;
use crate::shell::exec::builtins::mkdir;
use crate::shell::parse::Cmd;

use std::collections::HashMap;
use std::ffi::OsString;


use builtins::remove;
pub use builtins::{
    base::{echo, exit, pwd},
    cat, cd, list,
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
                        println!("Fork failed");
                        return;
                    } else if pid == 0 {
                        func(shell, &command);
                        // println!("Child process {} finished", getppid());
                        std::process::exit(0);
                    } else {
                        let mut status = 0;
                        wait(&mut status);
                        if status != 0 {
                            println!("error wait");
                        } else {
                            // println!("child {} finished", pid);
                        }
                    }
                }
            },
            None => {
                println!("Command not found: {}", command.exec);
            }
        }
    }
}

pub fn get_builtins() -> HashMap<String, fn(&mut Shell, &Cmd)> {
    HashMap::from([
        ("exit".to_string(), exit as fn(&mut Shell, &Cmd)),
        ("echo".to_string(), echo as fn(&mut Shell, &Cmd)),
        ("pwd".to_string(), pwd as fn(&mut Shell, &Cmd)),
        ("ls".to_string(), list::ls as fn(&mut Shell, &Cmd)), // chang ls signature
        ("cd".to_string(), cd::cd as fn(&mut Shell, &Cmd)),
        ("cat".to_string(), cat::cat as fn(&mut Shell, &Cmd)),
        ("mkdir".to_string(), mkdir::mkdir as fn(&mut Shell, &Cmd)),
        ("rm".to_string(), remove::rm as fn(&mut Shell, &Cmd)),
    ])
}
