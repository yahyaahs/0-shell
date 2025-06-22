mod tokenize;

pub use tokenize::*;

#[derive(Clone)]
pub struct Cmd {
    pub exec: String,
    pub flags: Vec<String>,
    pub args: Vec<String>,
}
