use crate::shell::Shell;
use std::{fs, io};

pub fn cat(_shell: &mut Shell, args: &Vec<String>) {
    if args.len() == 0 {
        let stdin = io::stdin();
        loop {
            let mut input = String::new();
            stdin.read_line(&mut input).unwrap();
            print!("{}", input);
        }
    } else {
        for file in args {
            let content = fs::read_to_string(file);
            match content {
                Ok(data) => print!("{}", data),
                Err(_) => println!("cat: {}: No such file or directory", file),
            };
        }
    }
}
