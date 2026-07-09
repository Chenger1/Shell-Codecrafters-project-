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
                let mut found = false;

                for key in self.store.keys(){
                    let value = self.store.get(key).unwrap();
                    let key_with_sign = format!("{}{}", "$", key);
                    if comm.contains(key_with_sign.as_str()){
                        let new_comm = comm.replace(key_with_sign.as_str(), value);
                        new_part.push(new_comm);
                        found = true;
                        break
                    }
                }
                if !found {
                    new_part.push(comm);
                }
            }
            new_vector.push(new_part);
        }

        new_vector
    }
}
