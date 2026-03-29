use std::{env, path};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const PATH: &str = "PATH";

pub struct FSUtils{}

impl FSUtils{
    pub fn new() -> Self{
        FSUtils {  }
    }

    fn is_file_executable(&self, file_name: &str) -> Option<bool>{
        std::fs::metadata(file_name).ok().map(|m| m.permissions().mode() & 0o111 != 0)
    }

    pub fn get_path_executables(&self) -> Vec<String>{
        let mut names: Vec<String> = vec![];
        match env::var_os(PATH) {
            Some(paths) => {
                for path in env::split_paths(&paths){
                    let Ok(dir) = fs::read_dir(path) else {continue;};
                    for res in dir{
                        let entry = res.unwrap();
                        let path = entry.path();
                        let path_name = path.to_string_lossy();
        
                        if self.is_file_executable(&path_name) == Some(true){
                            names.push(path.file_name().unwrap().to_os_string().into_string().unwrap());
                        }
                    }
                }
            },
            None => {
                println!("{PATH} is not defined in the environment.");
            }
        }
        
        names
    }

    pub fn is_executable(&self, input: &str) -> Option<String>{
        match env::var_os(PATH) {
            Some(paths) => {
                for path in env::split_paths(&paths) {
                    let joined = path.join(input);
                    let file_name = joined.to_str().unwrap();
                    
                    if self.is_file_executable(file_name) == Some(true){
                        return Some(file_name.to_string());
                    }
                }
            }
            None => {
                eprintln!("{PATH} is not defined in the environment.");
                return None;
            }
        }
        
        None
    }

    pub fn is_exist(&self, path: &str, absolute: bool) -> bool{
        let full_path;
        if absolute{
            full_path = path;
        }else{
            full_path = path;
        }
        Path::new(full_path).exists()
    }
}
