use crate::shell::State;

use super::*;

pub fn parse_command(input: &str) -> Result<(State, Cmd), String> {
    let exec = match input.split_whitespace().nth(0) {
        Some(exe) => exe.to_string(),
        None => return Err("".to_owned()),
    };

    let input = input.trim_start_matches(&exec).trim();

    // let final_parse:Result<> = ;
    Ok((
        State::Exec,
        Cmd {
            exec,
            args: tokenize(&input),
        },
    ))
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
