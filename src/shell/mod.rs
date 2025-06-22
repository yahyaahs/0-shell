use std::collections::HashMap;
use std::path::PathBuf;

use crate::shell::parse::Cmd;

pub mod exec;
pub mod parse;

#[allow(dead_code)] //to remove it later when all field is used

#[derive(Clone)]
pub struct Shell {
    pub pid: u32,                     // Shell's process ID
    pub cwd: PathBuf,                 // Current working directory
    pub env: HashMap<String, String>, // Environment variables
    pub history: Vec<String>,         // Command history
    pub last_status: i32,             // Exit status of the last command

    pub prompt: String,
    pub builtins: HashMap<String, fn(&mut Shell, &Cmd)>, // store all built in cmd
    pub state: State,
}

#[derive(Clone)]
pub enum State {
    Exec,
    Ready,
    Quote(String),
    BackNewLine,
}

use std::env;
use std::fs;
use std::path::Path;

pub fn update_prompt(shell: &mut Shell) {
    let display_path = shell.cwd.clone();
    let pwd_path = display_path.to_str().unwrap_or("");
    let home_dir = env::home_dir()
        .and_then(|p| p.to_str().map(|s| s.to_owned()))
        .unwrap_or_else(|| String::from(""));

    let last_segment = if pwd_path == home_dir {
        "~"
    } else {
        shell
            .cwd
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("")
    };

    let display_name = if last_segment.is_empty() {
        pwd_path.replace(&home_dir, "~")
    } else {
        last_segment.to_string()
    };

    let git_branch = get_git_branch(&shell.cwd);
    let branch_part = git_branch.map_or(String::new(), |b| {
        format!(" \x1b[31mgit:(\x1b[36m{}\x1b[31m)", b)
    });

    shell.prompt = format!(
        "\x1b[1m \x1b[32m{}{} \x1b[32m➜\x1b[0m ",
        display_name, branch_part
    );
}

fn get_git_branch(cwd: &Path) -> Option<String> {
    let git_path = find_git_dir(cwd)?;
    let head_path = git_path.join("HEAD");
    let head_content = fs::read_to_string(head_path).ok()?;

    if head_content.starts_with("ref: ") {
        let ref_path = head_content.trim_start_matches("ref: ").trim();
        let branch_name = ref_path.rsplit('/').next()?;
        Some(branch_name.to_string())
    } else {
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
        if candidate.is_file() {
            if let Ok(contents) = fs::read_to_string(&candidate) {
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
