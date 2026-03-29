use rustyline::completion::{Completer};
use prefix_tree_rs::Trie;

pub struct CommandLineHelper{
    builtin_prefix_tree: Trie,
    path_prefix_tree: Trie,
    builtin_command: [&'static str; 5],
    path_names: Vec<String>
}

impl CommandLineHelper{
    pub fn new(command: [&'static str; 5], path_executables: Vec<String>) -> CommandLineHelper{
        let mut trie = Trie::new();
        for word in command{
            trie.insert(word);
        }
        let mut path_trie = Trie::new();
        for exec in &path_executables{
            println!("{}", exec);
            path_trie.insert(exec.as_str());
        }

        CommandLineHelper { 
            builtin_prefix_tree:trie, 
            path_prefix_tree: path_trie, 
            builtin_command: command,
            path_names: path_executables
        }
    }

    fn get_builtin_command(&self, prefix: &str) -> Option<&'static str>{
        for word in self.builtin_command{
            if word.starts_with(prefix){
                return Some(word);
            }
        }
        None
    }

    fn get_path_command(&self, prefix: &str) -> Option<String>{
        for word in self.path_names.iter(){
            if word.starts_with(prefix){
                return Some(word.clone());
            }
        }
        None
    }
}

impl rustyline::Helper for CommandLineHelper{}
impl rustyline::highlight::Highlighter for CommandLineHelper {}
impl rustyline::validate::Validator for CommandLineHelper {}
impl rustyline::hint::Hinter for CommandLineHelper{
    type Hint = &'static str;
}

impl Completer for CommandLineHelper {
    type Candidate = String;

    fn complete(
            &self,
            line: &str,
            pos: usize,
            ctx: &rustyline::Context<'_>,
        ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
            if self.builtin_prefix_tree.starts_with(line){
                if let Some(found_word) = self.get_builtin_command(line){
                    let mut found_word = found_word.to_string().clone();
                    found_word.push(' ');
                    return Ok((0, vec![found_word]))
                }
                return Ok((0, vec![]))
            }

            if self.path_prefix_tree.starts_with(line){
                if let Some(mut found_word) = self.get_path_command(line){
                    found_word.push(' ');
                    return Ok((0, vec![found_word]))
                }
                return Ok((0, vec![]))
            }

        Ok((0, vec![]))   
    }
}
