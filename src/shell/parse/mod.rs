mod tokenize;

pub use tokenize::*;

pub struct Cmd {
    pub exec: String,
    pub args: Vec<String>, // it can be later a hashmap {"flag": "value_arg"}
}
