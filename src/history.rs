pub struct History{
    list_of_commands: Vec<String>,
}


impl History{
    pub fn new() -> Self{
        let list_of_commands = Vec::new();
        History{
            list_of_commands,
        }
    }

    pub fn add_command(&mut self, command: String){
        self.list_of_commands.push(command);
    }

    pub fn list_all_commands(&self) -> Vec<String>{
        self.list_of_commands.clone()
    }
}
