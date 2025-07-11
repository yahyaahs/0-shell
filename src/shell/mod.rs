use std::collections::HashMap;
use std::path::PathBuf;

use crate::shell::parse::Cmd;

pub mod exec;
pub mod parse;

#[derive(Clone, Debug)]
pub struct Shell {
    pub cwd: PathBuf, // Current working directory
    pub builtins: HashMap<String, fn(&mut Shell, &Cmd)>, // store all built in cmd
    pub state: State,
}

#[derive(Clone, Debug)]
pub enum State {
    Ready,
    Quote(String),
    BackNewLine,
}
