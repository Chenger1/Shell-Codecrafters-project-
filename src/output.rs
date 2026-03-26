use std::fs::File;
use std::io::Write;
use super::parser::Redirection;

pub struct Output{}

impl Output{
    pub fn new() -> Output{
        Output {  }
    } 

    fn redirect_to_file(&self, result: &String, path: &String, append: bool){
        let mut file = File::options()
        .write(!append)
        .append(append)
        .create(true)
        .open(path)
        .unwrap();

        file.write_all(result.as_bytes()).unwrap();
    }

    pub fn sdtout(&self, result: &String, redirection: &Redirection){
        match redirection{
            Redirection::RedirectStdout(path) => self.redirect_to_file(result, path, false),
            Redirection::AppendStdout(path) => self.redirect_to_file(result, path, true),
            _ => print!("{}", result)
        }
    }
    pub fn stderr(&self, result: &String, redirection: &Redirection){
        match redirection{
            Redirection::RedirectStdErr(path) => self.redirect_to_file(result, path, false),
            Redirection::AppendStderr(path) => self.redirect_to_file(result, path, true),
            _ => {
                if !result.is_empty(){
                    eprint!("{}", result)
                }
            }
        }
    }
}
