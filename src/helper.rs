use std::io;
use std::io::Write;
use std::{cell::RefCell, vec};
use rustyline::completion::{Completer};
use prefix_tree_rs::Trie;

pub struct CommandLineHelper{
    builtin_prefix_tree: Trie,
    path_prefix_tree: Trie,
    builtin_command: [&'static str; 5],
    path_names: Vec<String>,
    last_prompt: RefCell<Option<String>>
}

impl CommandLineHelper{
    pub fn new(command: [&'static str; 5], path_executables: Vec<String>) -> CommandLineHelper{
        let mut trie = Trie::new();
        for word in command{
            trie.insert(word);
        }
        let mut path_trie = Trie::new();
        for exec in &path_executables{
            path_trie.insert(exec.as_str());
        }

        CommandLineHelper { 
            builtin_prefix_tree:trie, 
            path_prefix_tree: path_trie, 
            builtin_command: command,
            path_names: path_executables,
            last_prompt: RefCell::new(None)
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

    fn get_all_path_commands(&self, prefix: &str) -> Vec<String>{
        let mut matches: Vec<String> = vec![];
        for word in self.path_names.iter(){
            if word.starts_with(prefix){
                matches.push(word.clone());
            }
        }
        matches.sort();
        matches
    }

    fn print_all_path_commands(&self, commands: &Vec<String>){
        println!("");
        for word in commands.iter(){
            print!("{}  ", word);
        }
        println!("");
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
            let mut result: Vec<String> = vec![];
            if self.builtin_prefix_tree.starts_with(line){
                if let Some(found_word) = self.get_builtin_command(line){
                    let mut found_word = found_word.to_string().clone();
                    found_word.push(' ');
                    result.push(found_word);
                }
            }
            else if self.path_prefix_tree.starts_with(line){
                let matched = self.get_all_path_commands(line);
                if matched.len() == 1{
                    let mut found_word = matched[0].clone();
                    println!("{}", found_word);
                    found_word.push(' ');
                    result.push(found_word);
                }else if matched.len() > 1 {
                    if let Some(last_prompted_word) = self.last_prompt.take(){
                        self.print_all_path_commands(&matched);
                        return Ok((0, vec![last_prompted_word]));
                    }else{
                        self.last_prompt.replace(Some(line.to_string()));
                        print!("\x07");
                        io::stdout().flush().unwrap();
                        return Ok((0, vec![line.to_string()]));
                    }
                }
            }
            self.last_prompt.replace(None);
            return Ok((0, result)); 
    }
}
