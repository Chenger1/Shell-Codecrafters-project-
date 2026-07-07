use std::collections::HashMap;


pub struct DeclaredVariables{
    store: HashMap<String, String>,
}


impl DeclaredVariables{
    pub fn new() -> Self{
        DeclaredVariables{store: HashMap::new()}
    }

    pub fn register(&mut self, command: String, description: String){
        self.store.insert(command, description);
    }

    pub fn get(&self, command: String) -> Option<&String> {
        self.store.get(&command)
    }
}
