
use std::collections::HashMap;

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

    pub fn get_completion(&self, command: &String) -> Option<&String>{
        self.completions.get(command)
    }
}
