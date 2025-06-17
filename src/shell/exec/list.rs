pub use std::fs;
use std::{ffi::OsString, fs::DirEntry, io::Error};
#[derive(Debug)]
pub enum Types {
    File(OsString),
    Dir(OsString),
    Executable(OsString),
    Symlink(OsString),
    NoSupported,
    Error,
}
pub fn check_type(name : DirEntry)-> Types{
    
    match name.metadata(){
        Ok(meta)=> 
            if meta.is_dir(){
                return Types::Dir(name.file_name());
            } else if meta.is_file() {
                return Types::File(name.file_name());
            } else if meta.is_symlink(){
                return Types::Symlink(name.file_name());
            } else {
                return Types::NoSupported;
            },
        _=>Types::Error,
    }

}
pub fn ls(args: &Vec<String>){
    let paths =fs::read_dir(".").unwrap();
    let mut types = vec![];
    for data in paths{
        types.push(check_type(data.unwrap()));
    }
    println!("types {:?}", types);
}