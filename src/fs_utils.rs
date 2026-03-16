use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub struct FSUtils{
    pub base_path: String
}

impl FSUtils{
    pub fn new() -> Self{
        FSUtils { base_path: String::from(".") }
    }

    pub fn is_executable(&self, input: &str) -> Option<String>{
        let key = "PATH";
        match env::var_os(key) {
            Some(paths) => {
                for path in env::split_paths(&paths) {
                    let joined = path.join(input);
                    let file_name = joined.to_str().unwrap();
                    
                    if Path::new(file_name).exists() {
                        if let Some(executable) = std::fs::metadata(file_name)
                        .ok()
                        .map(|m| m.permissions().mode() & 0o111 != 0)
                        {
                            if executable {
                                return Some(file_name.to_string());
                            }
                        }
                    }
                }
            }
            None => {
                println!("{key} is not defined in the environment.");
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
