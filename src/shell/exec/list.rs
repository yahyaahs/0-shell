pub use std::fs;
use std::fs::Metadata;

pub fn ls(args: &Vec<String>){
    let paths =fs::read_dir(".").unwrap();
    for path in paths{
        match path{
            Ok(name)=> 
            match name.metadata(){
                Ok(meta)=> if meta.is_dir(){
                    println!("directory {:?}", name.path());
                }else{
                    println!("{:?}", name.file_name())
                },
                _=>println!("error"),

            },
            _=> println!("no read"),
        }
    }
}