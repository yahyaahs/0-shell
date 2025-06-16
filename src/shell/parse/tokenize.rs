use super::*;

pub fn parse_command(input: &str) -> Cmd {
    let mut exec = String::new();
    let mut args: Vec<String> = Vec::new();
    let mut i = 0;
    for word in input.split(" ") {
        if i == 0 {
            exec = word.trim().to_string();
        } else {
            args.push(word.trim().to_string());
        }
        i += 1;
    }
    Cmd { exec, args }
}

// pub fn tokens(command: &String) -> Vec<Token> {
//     let mut tokens = Vec::new();
//     let mut current_token = String::new();
//     let mut in_quotes = false;

//     for c in command.chars() {
//         if c == '"' {
//             in_quotes = !in_quotes;
//         } else if c.is_whitespace() && !in_quotes {
//             if !current_token.is_empty() {
//                 tokens.push(current_token.clone());
//                 current_token.clear();
//             }
//         } else {
//             current_token.push(c);
//         }
//     }

//     if !current_token.is_empty() {
//         tokens.push(current_token);
//     }
//     let parsed_tokens = token_parsing(&tokens);
//     parsed_tokens
// }

// pub fn token_parsing(tokens: &Vec<String>) -> Vec<Token> {
//     let mut lexed_tokens = Vec::new();
//     for (i, token) in tokens.iter().enumerate() {
//         if token.starts_with('-') && i == 0 {
//             lexed_tokens.push(Token::Invalid(token.clone()));
//         } else if token.starts_with('-') {
//             lexed_tokens.push(Token::Arg(token.clone()));
//         } else {
//             lexed_tokens.push(Token::Command(token.clone()));
//         }
//     }
//     return lexed_tokens;
// }
