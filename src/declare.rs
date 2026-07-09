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
        commands
            .into_iter()
            .map(|part| {
                part.into_iter()
                    .map(|comm| self.substitute_in_token(&comm))
                    .filter(|comm| !comm.is_empty())
                    .collect()
            })
            .collect()
    }

    fn substitute_in_token(&self, comm: &str) -> String {
        let mut result = String::with_capacity(comm.len());
        let mut rest = comm;

        while let Some(dollar_pos) = rest.find('$') {
            result.push_str(&rest[..dollar_pos]);
            rest = &rest[dollar_pos + 1..];

            if let Some(braced) = rest.strip_prefix('{') {
                match braced.find('}') {
                    Some(end) => {
                        let name = &braced[..end];
                        // If not found, push nothing — the variable
                        // contributes an empty string, per shell semantics.
                        if let Some(value) = self.store.get(name) {
                            result.push_str(value);
                        }
                        rest = &braced[end + 1..];
                    }
                    None => {
                        // Unterminated brace — still no valid name to resolve,
                        // so this stays literal (not really a "variable" at all).
                        result.push_str("${");
                        rest = braced;
                    }
                }
            } else {
                let end = rest
                    .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .unwrap_or(rest.len());
                let name = &rest[..end];
                // Same here: missing variable expands to nothing.
                if let Some(value) = self.store.get(name) {
                    result.push_str(value);
                }
                rest = &rest[end..];
            }
        }

        result.push_str(rest);
        result
    }
}
