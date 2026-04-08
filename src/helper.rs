use std::io;
use std::io::Write;
use std::{cell::RefCell};
use rustyline::completion::{Completer, FilenameCompleter};
use rustyline::completion::Pair as RustyPair;
use prefix_tree_rs::Trie;

pub struct CommandLineHelper{
    builtin_prefix_tree: Trie,
    path_prefix_tree: Trie,
    builtin_command: [&'static str; 5],
    path_names: Vec<String>,
    last_prompt: RefCell<Option<String>>,
    filename_completer: FilenameCompleter
}

impl CommandLineHelper{
    pub fn new(command: [&'static str; 5], path_executables: Vec<String>) -> CommandLineHelper{
        let fc = FilenameCompleter::new();
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
            last_prompt: RefCell::new(None),
            filename_completer: fc
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

    fn find_longest_common_prefix(&self, words: &Vec<String>) -> String{
        if words.is_empty(){
            return String::new();
        }
        
        let mut prefix = words[0].clone();

        for word in &words[1..] {
            while !word.starts_with(prefix.as_str()) {
                prefix.pop();
                if prefix.is_empty() {
                    return String::new();
                }
            }
        }

        prefix
    }

}

impl rustyline::Helper for CommandLineHelper{}
impl rustyline::highlight::Highlighter for CommandLineHelper {}
impl rustyline::validate::Validator for CommandLineHelper {}
impl rustyline::hint::Hinter for CommandLineHelper{
    type Hint = &'static str;
}

impl Completer for CommandLineHelper {
    type Candidate = RustyPair;

    fn complete(
            &self,
            line: &str,
            pos: usize,
            ctx: &rustyline::Context<'_>,
        ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
            if line.split(" ").collect::<Vec<&str>>().len() > 1{
                return self.filename_completer.complete(line, pos, ctx);
            }

            let mut result: Vec<RustyPair> = vec![];
            if self.builtin_prefix_tree.starts_with(line){
                if let Some(found_word) = self.get_builtin_command(line){
                    let mut found_word = found_word.to_string().clone();
                    found_word.push(' ');
                    result.push(RustyPair {
                        display: found_word.clone(),
                        replacement: found_word,
                    });
                }
            }
            else if self.path_prefix_tree.starts_with(line){
                let matched = self.get_all_path_commands(line);
                if matched.len() == 1{
                    let mut found_word = matched[0].clone();
                    found_word.push(' ');
                    result.push(RustyPair {
                        display: found_word.clone(),
                        replacement: found_word,
                    });
                }else if matched.len() > 1 {
                    let prefix = self.find_longest_common_prefix(&matched);
                    if prefix.len() > line.to_string().len(){
                        return Ok((0, vec![RustyPair {
                            display: prefix.clone(),
                            replacement: prefix,
                        }]));
                    }

                    if let Some(_) = self.last_prompt.take(){
                        println!("\n{}", matched.join("  "));
                        return Ok((0, vec![RustyPair {
                            display: line.to_string(),
                            replacement: line.to_string(),
                        }]));
                    }else{
                        self.last_prompt.replace(Some(line.to_string()));
                        print!("\x07");
                        io::stdout().flush().unwrap();
                        return Ok((0, vec![RustyPair {
                            display: line.to_string(),
                            replacement: line.to_string(),
                        }]));
                    }
                }
            }
            self.last_prompt.replace(None);
            return Ok((0, result)); 
    }
}
