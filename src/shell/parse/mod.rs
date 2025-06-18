mod tokenize;

pub use tokenize::*;

pub struct Cmd {
    pub exec: String,
    pub flags: Vec<String>,
    pub args: Vec<String>,
}
