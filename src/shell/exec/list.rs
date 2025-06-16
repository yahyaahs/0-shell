pub use std::fs;

pub fn ls(args: &Vec<String>){
    let paths =fs::read_dir(".").unwrap();
    for path in paths{
        println!("paths {:?}", path);
    }
}