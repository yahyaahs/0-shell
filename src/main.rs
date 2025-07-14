mod shell;

use std::{
    env,
    io::{Read, stdin},
    path::PathBuf,
};

use shell::{
    Shell, State,
    exec::{execution, get_builtins},
    parse::{display_prompt, parse_command, scan_command},
};

use crate::shell::exec::builtins::write_;

unsafe extern "C" {
    fn signal(signal: i32, handler: extern "C" fn(i32));
    fn fcntl(fd: i32, cmd: i32, arg: i32) -> i32;
}

// fcntl constants (manual since no libc)
const F_GETFL: i32 = 3;
const F_SETFL: i32 = 4;
const O_NONBLOCK: i32 = 0x800;

const STDIN_FILENO: i32 = 0;

// Flush any bytes from stdin after Ctrl+C
fn clear_stdin_buffer() {
    unsafe {
        let flags = fcntl(STDIN_FILENO, F_GETFL, 0);
        fcntl(STDIN_FILENO, F_SETFL, flags | O_NONBLOCK);

        let mut buf = [0u8; 1024];
        let stdin = stdin();
        let mut handle = stdin.lock();

        while let Ok(n) = handle.read(&mut buf) {
            if n == 0 {
                break;
            }
        }

        fcntl(STDIN_FILENO, F_SETFL, flags); // restore blocking
    }
}

extern "C" fn signal_handler(_signal: i32) {
    // match stdout().flush(){
    //     Ok(_) => {}
    //     Err(err) => std::process::exit(1),
    // };
    // print!("\n{}", display_prompt());
    // match stdout().flush(){
    //     Ok(_) => {}
    //     Err(err) => std::process::exit(1),
    // };
    clear_stdin_buffer(); // discard partial input

    write_("\n");
    write_(&display_prompt());
    unsafe { INIT_SHELL = true };
}

static mut INIT_SHELL: bool = false;

fn main() {
    unsafe {
        signal(2, signal_handler);
    }

    let mut shell = Shell {
        cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
        builtins: get_builtins(),
        state: shell::State::Ready,
    };

    let stdin = stdin();
    let mut input = String::new();

    loop {
        match &shell.state {
            State::Ready => {
                write_(&display_prompt());
                input = String::new();
            }
            State::Quote(typ) => {
                write_(&format!("{}> ", typ));
                shell.state = State::Ready;
            }
            State::BackNewLine => {
                write_(">");
                shell.state = State::Ready;
            }
        };

        let is_empty = if input.len() > 0 {
            let mut temp_buffer = String::new();
            match stdin.read_line(&mut temp_buffer) {
                Ok(byt) => {
                    if byt == 0 {
                        write_("\nexiting shell...\n");
                        return;
                    }
                }
                Err(err) => write_(&format!("shell error: {}\n", err.to_string())),
            };
            input = format!("{}{}", input, temp_buffer);
            temp_buffer.len() == 1
        } else {
            match stdin.read_line(&mut input) {
                Ok(byt) => {
                    if byt == 0 {
                        write_("\nexiting shell...\n");
                        return;
                    }
                }
                Err(err) => write_(&format!("shell error: {}\n", err.to_string())),
            };
            input.len() == 1
        };

        unsafe {
            if INIT_SHELL {
                shell.state = State::Ready;
                println!("{}", input);
                INIT_SHELL = false;
                continue;
            }
        }

        let mut to_scan = input.trim().to_string();
        let state = scan_command(&mut to_scan, is_empty);
        input = to_scan;

        match state {
            Some(new_state) => shell.state = new_state,
            None => match parse_command(&input) {
                Ok(cmd) => {
                    execution(&mut shell, cmd);
                }
                Err(err) => write_(&format!("{}", err)),
            },
        };
    }
}
