mod tokenize;

pub use tokenize::*;

#[derive(Debug)]
pub struct Cmd {
    pub exec: String,
    pub flags: Vec<String>,
    pub args: Vec<String>,
}
