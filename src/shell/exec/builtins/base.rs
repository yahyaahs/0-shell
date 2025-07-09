use super::*;

pub fn exit(_shell: &mut Shell, cmd: &Cmd) {
    if cmd.args.len() == 0 {
        std::process::exit(0)
    };
    match cmd.args[0].parse::<i32>() {
        Ok(nb) => std::process::exit(nb),
        Err(_) => std::process::exit(0),
    };
}

pub fn echo(_shell: &mut Shell, cmd: &Cmd) {
    println!("{}", cmd.args.join(" "));
}

pub fn pwd(shell: &mut Shell, _cmd: &Cmd) {
    println!("{}", shell.cwd.to_str().unwrap_or(""));
}
