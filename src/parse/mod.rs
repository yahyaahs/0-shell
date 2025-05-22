use std::io::{self, Write};
pub fn check_command() -> String {
    let mut command = String::new();

    for line in io::stdin().lines() {
        let line = line.unwrap();
        if line.trim() == "exit" {
            return "Exit".to_string();
        }
        if line.trim().chars().last() == Some('\\'){
            command.push_str(&line.trim()[..line.len()-1]);
            command.push(' ');
            print!("> ");
            io::stdout().flush().unwrap();
            continue;
        } else {
            command.push_str(&line.trim());
            break;
        }
    }
    return command;
}
