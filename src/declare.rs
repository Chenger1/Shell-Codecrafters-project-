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
            .map(|part| part.into_iter().map(|comm| self.substitute_in_token(&comm)).collect())
            .collect()
    }

    fn substitute_in_token(&self, comm: &str) -> String {
        let mut result = String::with_capacity(comm.len());
        let mut chars = comm.char_indices().peekable();

        while let Some((i, c)) = chars.next() {
            if c != '$' {
                result.push(c);
                continue;
            }

            // Check what follows '$' to decide which syntax we're parsing.
            if let Some(&(brace_start, '{')) = chars.peek() {
                // Braced form: ${name} — consume the '{'.
                chars.next();
                let name_start = brace_start + 1; // just after '{'
                let mut name_end = name_start;
                let mut closed = false;

                // Scan until we hit '}', consuming everything in between
                // as the variable name (no character-class restriction needed,
                // since '}' unambiguously marks the end).
                while let Some(&(j, ch)) = chars.peek() {
                    if ch == '}' {
                        chars.next(); // consume the closing brace
                        closed = true;
                        break;
                    }
                    name_end = j + ch.len_utf8();
                    chars.next();
                }

                if closed {
                    let var_name = &comm[name_start..name_end];
                    match self.store.get(var_name) {
                        Some(value) => result.push_str(value),
                        // Not found — keep the original "${name}" text unchanged.
                        None => {
                            result.push_str("${");
                            result.push_str(var_name);
                            result.push('}');
                        }
                    }
                } else {
                    // No closing brace found before the string ended — treat
                    // as literal text rather than a valid substitution, since
                    // there's no well-formed variable name to resolve.
                    result.push_str("${");
                    result.push_str(&comm[name_start..name_end]);
                }

                continue;
            }

            // Unbraced form: $name — same as before, greedily consume
            // identifier characters (letters, digits, underscore).
            let start = i + 1;
            let mut end = start;

            while let Some(&(j, ch)) = chars.peek() {
                if ch.is_alphanumeric() || ch == '_' {
                    end = j + ch.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }

            let var_name = &comm[start..end];
            match self.store.get(var_name) {
                Some(value) => result.push_str(value),
                None => {
                    result.push('$');
                    result.push_str(var_name);
                }
            }
        }

        result
    }
}
