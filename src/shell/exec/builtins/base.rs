use super::*;

pub fn exit(_shell: &mut Shell, cmd: &Cmd) {
    if cmd.args.len() == 0 {
        std::process::exit(0)
    };
    match cmd.args[0].parse::<i32>() {
        Ok(nb) => {
            let code = nb.clamp(0, 255);
            std::process::exit(code)
        }
        Err(_) => {
            println!("exit {}: undefined code", cmd.args[0]);
            return;
        }
    };
}

pub fn echo(_shell: &mut Shell, cmd: &Cmd) {
    println!("{}", cmd.args.join(" "));
}

pub fn pwd(shell: &mut Shell, _cmd: &Cmd) {
    println!(
        "{}",
        shell
            .cwd
            .to_str()
            .unwrap_or("cannot find current directory")
    );
}
