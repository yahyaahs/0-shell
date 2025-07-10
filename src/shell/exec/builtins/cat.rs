use super::*;

use std::{fs, io};

// unsafe extern "C" {
//     fn signal(signal: i32, handler: extern "C" fn(i32));
// }
// extern "C" fn signal_handler(_signal: i32) {
//     println!("\nsignal, exit");
//     // std::process::exit(0);
// }
pub fn cat(_shell: &mut Shell, cmd: &Cmd) {
    // unsafe {
    // signal(2, signal_handler);
    // }
    if cmd.args.len() == 0 {
        let stdin = io::stdin();
        loop {
            let mut input = String::new();
            let  bytes = stdin.read_line(&mut input).unwrap();
            if bytes == 0 {
                println!();
                break; 
            }
            print!("{}", input);
        }
    } else {
        for file in cmd.args.clone() {
            let content = fs::read_to_string(file.clone());
            match content {
                Ok(data) => print!("{}", data),
                Err(_) => println!("cat: {}: No such file or directory", file),
            };
        }
    }
}
