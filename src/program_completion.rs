use std::collections::HashMap;
use std::process::Command;

pub struct ProgrammableCompletor{
    completions: HashMap<String, String>
}


impl ProgrammableCompletor{
    pub fn new() -> ProgrammableCompletor{
        let mut completions = HashMap::new();
        ProgrammableCompletor { completions }
    }

    pub fn register_completion(&mut self, command: String, path: String){
        self.completions.insert(command, path);
    }

    pub fn get_completion_info(&self, command: &String) -> Option<&String>{
        self.completions.get(command)
    }

    pub fn get_candidates(&self, command: &String) -> Option<Vec<String>>{
        if !command.ends_with(" "){
            return None;
        }

        let file_path = self.completions.get(command.trim());
        let mut completions: Vec<String> = vec!{};
        if let Some(path) = file_path{
            let output = Command::new(path).output().expect("failed to execute process");
            let res = String::from_utf8_lossy(&output.stdout);
            for candidate in res.split("\n"){
                if candidate == ""{
                    continue
                }
                completions.push(candidate.to_string().trim().to_string());
            }
        }
        Some(completions)
    }
}
