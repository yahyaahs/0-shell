mod shell;

use std::{
    env,
    io::{Write, stdin},
    path::PathBuf,
    io
};

use shell::{
    Shell, State,
    exec::{execution, get_builtins},
    parse::{display_prompt, parse_command, scan_command},
};

use crate::shell::exec::builtins::write_;

unsafe extern "C" {
    fn signal(signal: i32, handler: extern "C" fn(i32));
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
    write_("\n");
    write_(&display_prompt());
}

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
            }
            State::BackNewLine => {
                write_(">");
            }
        };

        if input.len() > 0 {
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
        }

        let input = input.trim();
        let state = scan_command(&input.trim());
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
