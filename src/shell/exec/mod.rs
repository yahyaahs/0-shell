pub mod builtins;
pub mod helper;

use std::{
    sync::Arc,
    thread::{JoinHandle, spawn},
};

use super::Shell;
use crate::shell::{exec::helper::find_non_builtins, parse::Cmd};

use std::sync::Mutex;

pub struct StackData {
    pub processes: Vec<JoinHandle<()>>,
}

pub fn add_process(stack: &Mutex<StackData>, handle: JoinHandle<()>) {
    let mut guard = stack.lock().unwrap();
    guard.processes.push(handle);
}

pub fn join_all(stack: &Mutex<StackData>) {
    let mut guard = stack.lock().unwrap();
    while let Some(handle) = guard.processes.pop() {
        match handle.join() {
            Ok(_) => continue,
            Err(_) => {
                println!("da3na awald 3ami")
            }
        };
    }
}

pub fn execute_command(shell: Arc<Mutex<Shell>>, stack: &Mutex<StackData>, command: Cmd) {
    let shell_clone = Arc::clone(&shell);
    let command_clone = command.clone();

    let handle = spawn(move || {
        let mut shell_locked = match shell_clone.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };

        match shell_locked.builtins.get(&command_clone.exec) {
            Some(func) => func(&mut shell_locked, &command_clone),
            None => {
                println!("Command not found: {}", command_clone.exec);

                let bin_cmd = find_non_builtins(&command.exec);
                if let Some(bin) = bin_cmd {
                    println!("but we found this: {}", bin);
                }
            }
        }
    });

    add_process(stack, handle);
}
