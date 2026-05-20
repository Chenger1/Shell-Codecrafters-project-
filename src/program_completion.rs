use std::collections::HashMap;
use std::process::Command;
use std::env;

pub struct ProgrammableCompletor{
    completions: HashMap<String, String>
}


impl ProgrammableCompletor{
    pub fn new() -> ProgrammableCompletor{
        let completions = HashMap::new();
        ProgrammableCompletor { completions }
    }

    pub fn register_completion(&mut self, command: String, path: String){
        self.completions.insert(command, path);
    }

    pub fn get_completion_info(&self, command: &String) -> Option<&String>{
        self.completions.get(command)
    }

    pub fn get_candidates(&self, command: &String, original_position: &usize) -> Option<(Vec<String>, usize)>{
        if !command.ends_with(" ") && !command.contains(" "){
            return None;
        }
        let words: Vec<&str> = command.split(" ").collect();
        let arg_1 = words[0].to_string();
        let file_path = self.completions.get(arg_1.trim());
        if file_path.is_none(){
            return None;
        }
        
        let mut pos = command.len();
        let mut completions: Vec<String> = vec!{};
        if let Some(path) = file_path{
            unsafe{
                env::set_var("COMP_LINE", command);
                env::set_var("COMP_POINT", original_position.to_string());
            }

            let arg_2 = words.last().unwrap().to_string();
            pos -= arg_2.len();
            let mut arg_3= String::from("");
            if words.len() > 2{
                arg_3 = words[words.len()-2].to_string();
            }
            let args: Vec<String> = vec![arg_1, arg_2, arg_3];

            let output = Command::new(path).args(args).output().expect("failed to execute process");
            let res = String::from_utf8_lossy(&output.stdout);
            for candidate in res.split("\n"){
                if candidate == ""{
                    continue
                }
                completions.push(candidate.to_string().trim().to_string());
            }
        }
        Some((completions, pos))
    }
}
