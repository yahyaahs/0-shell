use std::io;
use std::io::Write;

use check::check_command;
use tokenize::tokens;

mod check;
mod tokenize;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let command = check_command();
        if command == "exit"  {
            break;
        }
        let tokens = tokens(&command);
        println!("$ {:?}", tokens);
    }
}
