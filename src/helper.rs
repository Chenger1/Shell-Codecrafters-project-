use std::io;
use std::io::Write;
use std::{cell::RefCell};
use rustyline::completion::{Completer, FilenameCompleter};
use rustyline::completion::Pair as RustyPair;
use prefix_tree_rs::Trie;
use rustyline::hint::HistoryHinter;

use crate::program_completion::ProgrammableCompletor;

pub struct CommandLineHelper{
    builtin_prefix_tree: Trie,
    path_prefix_tree: Trie,
    builtin_command: [&'static str; 8],
    path_names: Vec<String>,
    last_prompt: RefCell<Option<String>>,
    filename_completer: FilenameCompleter,
    first_tab: RefCell<bool>,
    #[allow(unused)]
    hinter: HistoryHinter,
    pub programmable_completor: ProgrammableCompletor
}

impl CommandLineHelper{
    pub fn new(command: [&'static str; 8], path_executables: Vec<String>) -> CommandLineHelper{
        let fc = FilenameCompleter::new();
        let mut trie = Trie::new();
        for word in command{
            trie.insert(word);
        }
        let mut path_trie = Trie::new();
        for exec in &path_executables{
            path_trie.insert(exec.as_str());
        }
        let programmable_completor = ProgrammableCompletor::new();

        CommandLineHelper { 
            builtin_prefix_tree:trie, 
            path_prefix_tree: path_trie, 
            builtin_command: command,
            path_names: path_executables,
            last_prompt: RefCell::new(None),
            filename_completer: fc,
            first_tab: RefCell::new(false),
            hinter: HistoryHinter::new(),
            programmable_completor
        }
    }

    pub fn clear_state(&mut self){
        self.first_tab.replace(false);
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

    fn filename_completion(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>,) -> Option<rustyline::Result<(usize, Vec<<CommandLineHelper as Completer>::Candidate>)>>{
        let splitted_line = line.split(" ").collect::<Vec<&str>>();
        if splitted_line.len() > 1{
            self.last_prompt.replace(None);
            let result = self.filename_completer.complete(line, pos, ctx);
            if let Ok((start, candidates)) = result{
                let updated_candidates: Vec<RustyPair> = candidates.into_iter().map(|mut candidate|{
                    if !candidate.replacement.ends_with("/"){
                        candidate.replacement.push(' ');
                    }
                    candidate.display = candidate.replacement.clone();
                    candidate
                }).collect();
                if updated_candidates.len() == 1{
                    return Some(Ok((start, vec![updated_candidates[0].clone()])));
                }
                let candidates_strings: Vec<String> = updated_candidates.clone().into_iter().map(|candidate| candidate.replacement).collect();
                let prefix = self.find_longest_common_prefix(&candidates_strings);
                if prefix.len() > splitted_line[1].to_string().len(){
                    return Some(Ok((splitted_line[0].to_string().len() + 1, vec![RustyPair{
                        display: prefix.clone(),
                        replacement: prefix
                    }])))
                }
                if !self.first_tab.take(){
                    self.first_tab.replace(true);
                    print!("\x07");
                    let result = vec![];
                    return Some(Ok((0, result))); 
                }else{
                    self.first_tab.replace(true);
                    return Some(Ok((start, updated_candidates)));
                }
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

fn make_pair(s: String) -> RustyPair{
    let mut s = s;
    s.push(' ');
    RustyPair {display: s.clone(), replacement: s}
}

impl Completer for CommandLineHelper {
    type Candidate = RustyPair;

    fn complete(
            &self,
            line: &str,
            pos: usize,
            ctx: &rustyline::Context<'_>,
        ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
            let candidates = self.programmable_completor.get_candidates(&line.to_string());
            if let Some(candidates) = candidates{
                if candidates.len() == 0{
                    print!("\x07");
                    io::stdout().flush().unwrap();
                    return Ok((0, vec![RustyPair {
                        display: line.to_string(),
                        replacement: line.to_string(),
                    }]));
                }
                let mut replacement = candidates[0].clone();
                replacement.push(' ');
                return Ok((pos, vec![RustyPair {
                        display: replacement.clone(),
                        replacement: replacement,
                    }]));
            }

            if let Some(result) = self.filename_completion(line, pos, ctx){
                return result
            }
            let mut result: Vec<RustyPair> = vec![];
            if self.builtin_prefix_tree.starts_with(line){
                if let Some(found_word) = self.get_builtin_command(line){
                    let found_word = found_word.to_string().clone();
                    result.push(make_pair(found_word));
                }
            }
            else if self.path_prefix_tree.starts_with(line){
                let matched = self.get_all_path_commands(line);
                if matched.len() == 1{
                    let found_word = matched[0].clone();
                    result.push(make_pair(found_word));
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
