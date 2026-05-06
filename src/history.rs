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

    pub fn list_all_commands(&self) -> VecDeque<(usize, String)>{
        self.list_of_commands.clone()
    }
}
