use std::io;
use std::io::Write;

use parse::check_command;

mod parse;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let command = check_command();
        if command == "exit"  {
            break;
        }
        println!("$ {}", command);
    }
}
