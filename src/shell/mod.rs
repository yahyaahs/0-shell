use std::collections::HashMap;
use std::path::PathBuf;

use crate::shell::parse::Cmd;

pub mod exec;
pub mod parse;

#[derive(Clone)]
pub struct Shell {
    pub cwd: PathBuf, // Current working directory
    pub prompt: String,
    pub builtins: HashMap<String, fn(&mut Shell, &Cmd)>, // store all built in cmd
    pub state: State,
}

#[derive(Clone)]
pub enum State {
    Ready,
    Quote(String),
    BackNewLine,
}
