use std::collections::HashMap;


pub struct DeclaredVariables{
    store: HashMap<String, String>,
}


impl DeclaredVariables{
    pub fn new() -> Self{
        DeclaredVariables{store: HashMap::new()}
    }

    pub fn validate_command(&self, command: &str) -> bool{
        let first_ok = match command.chars().next(){
            Some(c) => c.is_alphabetic() || c == '_',
            None => false
        };
        first_ok && command.chars().all(|c| c.is_alphanumeric() || c == '_')

    }

    pub fn register(&mut self, command: String, description: String){
        self.store.insert(command, description);
    }

    pub fn get(&self, command: &String) -> Option<&String> {
        self.store.get(command)
    }

    pub fn substitute_variables(&self, commands: Vec<Vec<String>>) -> Vec<Vec<String>> {
        let mut new_vector: Vec<Vec<String>> = Vec::new();

        for part in commands {
            let mut new_part: Vec<String> = Vec::new();

            for comm in part {
                if !comm.starts_with("$") {
                    new_part.push(comm);
                    continue;
                }

                let comm = comm.replace("$", "").trim().to_string();
                let description = self.get(&comm);
                if let Some(description) = description {
                    new_part.push(description.clone());
                } else {
                    new_part.push(comm);
                }
            }
            new_vector.push(new_part);
        }

        new_vector
    }
}
