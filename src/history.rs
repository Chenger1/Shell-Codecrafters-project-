use std::collections::VecDeque;

pub struct History{
    list_of_commands: VecDeque<(usize, String)>,
}


impl History{
    pub fn new() -> Self{
        let list_of_commands = VecDeque::new();
        History{
            list_of_commands,
        }
    }

    pub fn add_command(&mut self, command: String){
        let index = self.list_of_commands.len();
        self.list_of_commands.push_front((index, command));
    }

    pub fn commands(&self) -> Box<dyn Iterator<Item=&(usize, String)> + '_>{
        Box::new(self.list_of_commands.iter().rev())
    }

    pub fn last_n(&self, n: usize) -> Box<dyn Iterator<Item=&(usize, String)> + '_>{
        let skip = self.list_of_commands.len().saturating_sub(n);
        Box::new(self.list_of_commands.iter().rev().skip(skip))
    }
}
