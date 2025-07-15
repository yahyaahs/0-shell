use std::io::{self, Write};

pub mod base;
pub mod cat;
pub mod cd;
pub mod list;
pub mod mkdir;
pub mod remove;
pub mod mv;
pub mod copy;

pub use super::*;

pub fn write_(s: &str) {
    let mut stdout = io::stdout();
    match stdout.write_all(s.as_bytes()) {
        Ok(_) => {}
        Err(_) => {
            std::process::exit(1);
        }
    }
    match stdout.flush() {
        Ok(_) => {}
        Err(_) => {
            std::process::exit(1);
        }
    };
}
