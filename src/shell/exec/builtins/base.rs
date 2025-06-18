use crate::shell::Shell;

pub fn exit(_shell: &mut Shell, args: &Vec<String>) {
    if args.len() == 0 {
        std::process::exit(0)
    };
    match args[0].parse::<i32>() {
        Ok(nb) => std::process::exit(nb),
        Err(_) => std::process::exit(0),
    };
}

pub fn echo(_shell: &mut Shell, args: &Vec<String>) {
    println!("{}", args.join(" "));
}
