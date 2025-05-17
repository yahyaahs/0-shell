use std::io;
use std::io::Write;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let command = input.trim();
        
        print!("{}\n", command);


        if command == "exit" {
            break;
        }
    }
}
