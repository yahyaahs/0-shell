use super::Shell;
use super::exec::*;
use super::parse::*;

use std::env;
use std::fs;
use std::io::Write;
use std::io::{stdin, stdout};
use std::path::{Path, PathBuf};

use crate::shell::State;
use crate::shell::exec::builtins::get_builtins;

impl Shell {
    pub fn new() -> Shell {
        Shell {
            pid: std::process::id(),
            cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            env: env::vars().collect(),
            history: Vec::new(),
            last_status: 0,

            prompt: "$ ".to_string(),
            builtins: get_builtins(),
            state: State::Ready,
        }
    }

    pub fn update_prompt(&mut self) {
        let mut display_path = self.cwd.clone();
        display_path = PathBuf::from("~").join(display_path);

        // Get Git branch name, if any
        let git_branch = get_git_branch(&self.cwd);
        let branch_part = git_branch.map_or(String::new(), |b| format!(" \x1b[31mgit:(\x1b[36m{}\x1b[31m)", b));

        self.prompt = format!(
            "\x1b[1m \x1b[32m{}{} \x1b[32m➜\x1b[0m ",
            display_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(""),
            branch_part
        );
    }

    pub fn run(mut self) {
        self.update_prompt();
        let stdin = stdin();
        let mut input = String::new();

        loop {
            match &self.state {
                State::Exec => continue,
                State::Ready => {
                    print!("{}", self.prompt);
                    stdout().flush().unwrap();
                    input = String::new();
                }
                State::Quote(typ) => {
                    print!("{}> ", typ);
                    self.state = State::Ready;
                    stdout().flush().unwrap();
                }
                State::BackNewLine => {
                    print!("> ");
                    self.state = State::Ready;
                    stdout().flush().unwrap();
                }
            };

            if input.len() > 0 {
                let mut temp_buffer = String::new();
                stdin.read_line(&mut temp_buffer).unwrap();
                input = format!("{}{}", input, temp_buffer);
            } else {
                stdin.read_line(&mut input).unwrap();
            }

            let input = input.trim();
            let state = scan_command(&input.trim());
            match state {
                State::Exec => match parse_command(&input) {
                    Ok((state, cmd)) => {
                        println!("to exec: [{}] [{:?}]", cmd.exec, cmd.args);
                        match state {
                            State::Exec => {
                                self.state = State::Exec;
                                execute_command(&mut self, &cmd);
                            }
                            _ => self.state = state,
                        }
                    }
                    Err(err) => print!("{err}"),
                },
                _ => self.state = state,
            };
        }
    }
}

fn get_git_branch(cwd: &Path) -> Option<String> {
    // Find the .git directory, which might be a folder or a file (in case of submodules or worktrees)
    let git_path = find_git_dir(cwd)?;

    let head_path = git_path.join("HEAD");

    let head_content = fs::read_to_string(head_path).ok()?;

    // Typical content:
    // ref: refs/heads/main
    if head_content.starts_with("ref: ") {
        let ref_path = head_content.trim_start_matches("ref: ").trim();
        let branch_name = ref_path.rsplit('/').next()?;
        Some(branch_name.to_string())
    } else {
        // Detached HEAD or commit hash, just return first 7 chars as a short SHA
        Some(head_content.trim().chars().take(7).collect())
    }
}

// Recursively look for .git folder
fn find_git_dir(start: &Path) -> Option<PathBuf> {
    let mut current = start;

    loop {
        let candidate = current.join(".git");
        if candidate.is_dir() {
            return Some(candidate);
        }
        // Sometimes .git is a file pointing to another git dir (git worktrees etc)
        if candidate.is_file() {
            // Read the gitdir path inside the file
            if let Ok(contents) = fs::read_to_string(&candidate) {
                // Usually something like: gitdir: /path/to/actual/git/dir
                if contents.starts_with("gitdir: ") {
                    let path_str = contents["gitdir: ".len()..].trim();
                    let gitdir_path = if Path::new(path_str).is_absolute() {
                        PathBuf::from(path_str)
                    } else {
                        current.join(path_str)
                    };
                    return Some(gitdir_path);
                }
            }
        }
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }
    None
}
