use rustyline::completion::{Completer};
use prefix_tree_rs::Trie;

pub struct CommandLineHelper{
    prefix_tree: Trie,
    builtin_command: [&'static str; 5]
}

impl CommandLineHelper{
    pub fn new(command: [&'static str; 5]) -> CommandLineHelper{
        let mut trie = Trie::new();
        for word in command{
            trie.insert(word);
        }

        CommandLineHelper { prefix_tree:trie, builtin_command: command }
    }

    fn get_word_from_tree(&self, prefix: &str) -> Option<&'static str>{
        for word in self.builtin_command{
            if word.starts_with(prefix){
                return Some(word);
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
            &self, // FIXME should be `&mut self`
            line: &str,
            pos: usize,
            ctx: &rustyline::Context<'_>,
        ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
            if self.prefix_tree.starts_with(line){
                if let Some(found_word) = self.get_word_from_tree(line){
                    let mut found_word = found_word.to_string().clone();
                    found_word.push(' ');
                    return Ok((0, vec![found_word]))
                }
                return Ok((0, vec![]))
            }
        Ok((0, vec![]))   
    }
}
