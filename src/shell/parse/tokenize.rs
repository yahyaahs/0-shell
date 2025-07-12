use super::*;

use crate::shell::State;
use chrono::Local;

pub fn scan_command(input: &str) -> Option<State> {
    if input.ends_with("\\") && !input.ends_with("\\\\") {
        return Some(State::BackNewLine);
    }

    let mut in_quote = None;
    let mut escaped = false;

    for c in input.chars() {
        if escaped {
            escaped = false;
            continue;
        }

        match c {
            '\\' => {
                escaped = true;
            }
            '"' | '\'' => match in_quote {
                Some(q) if q == c => in_quote = None,
                None => in_quote = Some(c),
                _ => {}
            },
            _ => {}
        }
    }

    if let Some(q) = in_quote {
        if q == '\"' {
            return Some(State::Quote("dquote".to_string()));
        } else {
            return Some(State::Quote("quote".to_string()));
        }
    }

    None
}

pub fn parse_command(input: &str) -> Result<Cmd, String> {
    let exec = match input.split_whitespace().nth(0) {
        Some(exe) => exe.to_string(),
        None => return Err("".to_owned()),
    };

    let input = input.trim_start_matches(&exec).trim();

    let all_tokens = tokenize(&input);
    let mut args: Vec<String> = Vec::new();
    let mut flags: Vec<String> = Vec::new();

    for arg in all_tokens {
        if arg.starts_with('-') {
            let new_vec: Vec<String> = arg
                .trim_start_matches('-')
                .chars()
                .map(|c| c.to_string())
                .collect();

            if valid_flags(&exec, &new_vec) {
                new_vec.iter().for_each(|fl| flags.push(fl.to_owned()));
            } else {
                return Err(format!(
                    "{}: invalid option -- '{}'\n",
                    exec,
                    arg.trim_start_matches('-')
                ));
            }
        } else {
            args.push(arg.to_owned());
        }
    }

    Ok(Cmd { exec, flags, args })
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();

    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(&ch) = chars.peek() {
        match ch {
            '\\' => {
                chars.next(); // consume '\'
                if let Some(&escaped_char) = chars.peek() {
                    current.push(escaped_char);
                    chars.next(); // consume escaped char
                }
            }
            '\'' => {
                chars.next(); // consume quote
                if !in_double_quote {
                    in_single_quote = !in_single_quote;
                } else {
                    current.push(ch);
                }
            }
            '"' => {
                chars.next(); // consume quote
                if !in_single_quote {
                    in_double_quote = !in_double_quote;
                } else {
                    current.push(ch);
                }
            }
            ' ' | '\t' => {
                if in_single_quote || in_double_quote {
                    current.push(ch);
                    chars.next();
                } else {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                    while let Some(&space) = chars.peek() {
                        if space == ' ' || space == '\t' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
            }
            _ => {
                current.push(ch);
                chars.next();
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn valid_flags(exec: &str, args: &Vec<String>) -> bool {
    match exec {
        "ls" => {
            args.iter()
                .filter(|f| {
                    **f != "l".to_string() && **f != "F".to_string() && **f != "a".to_string()
                })
                .collect::<Vec<&String>>()
                .len()
                == 0
        }
        "rm" => {
            args.iter()
                .filter(|f| **f != "r".to_string())
                .collect::<Vec<&String>>()
                .len()
                == 0
        }
        _ => args.len() == 0,
    }
}

pub fn display_prompt() -> String {
    let time = Local::now().format("%H:%M:%S");

    format!(
        "{magenta}╭─[{white}{blue}{time}{reset}{magenta}]{reset}\n{magenta}╰─» {reset}",
        magenta = "\x1b[36m",
        white = "\x1b[1;37m",
        blue = "\x1b[94m",
        reset = "\x1b[0m",
        time = time
    )
}
