use std::{env, fs};
#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::Command;
use std::path::Path;

pub mod fs_utils;

const BUILTIN_COMMANDS: [&'static str; 5] = ["echo", "exit", "type", "pwd", "cd"];

struct ShellCommand {
    fs_utils: fs_utils::FSUtils
}

impl ShellCommand {
    pub fn new() -> Self{
        let utils = fs_utils::FSUtils::new();
        ShellCommand { fs_utils: utils}
    }
    
    pub fn echo(&self, input: Vec<String>){
        let str_ = input.join(" ");
        println!("{}", str_);
    }

    pub fn pwd(&self){
        let path = fs::canonicalize(".").unwrap();
        println!("{}", path.to_str().unwrap());
    }

    pub fn execute(&self, input: &String, arguments: Vec<String>) -> io::Result<()>{
        let executable_file_name = self.fs_utils.is_executable(&input);
        if let Some(_) = executable_file_name {
            let output = Command::new(input).args(arguments).output()?;
            if output.status.success() {
                io::stdout().write_all(&output.stdout)?;
            } else {
                io::stderr().write_all(&output.stderr)?;
            }
        } else {
            println!("{}: command not found", input);
        }
        Ok(())
    }

    pub fn type_(&self, input: &String) -> io::Result<()>{
        if BUILTIN_COMMANDS.contains(&input.as_str()) {
            println!("{} is a shell builtin", input);
            return Ok(());
        };
        
        let executable_file_name = self.fs_utils.is_executable(&input);
        if let Some(file_name) = executable_file_name {
            println!("{} is {}", input, file_name);
        } else {
            println!("{} not found", input);
        }
        Ok(())
    }

    pub fn cd(&mut self, path: &String) -> std::io::Result<()>{
        let mut desired_path = path.to_string();
        if path == "~"{
            let home = env::var("HOME").unwrap();
            desired_path = home;
        }

        let is_absolute = Path::new(&desired_path).is_absolute();
        if !self.fs_utils.is_exist(&desired_path, is_absolute){
            println!("cd: {}: No such file or directory", desired_path);
            return Ok(())
        }

        let new_dir = Path::new(&desired_path);
        env::set_current_dir(&new_dir)?;
        Ok(())
    }

}

// utils
fn parse_arguments(input: &String) -> Vec<String> {
    let cleaned = input.trim();
    let mut commands: Vec<String> = vec![];
    let mut new_input: Vec<char> = vec![];
    let mut is_quoted = false;
    let mut whitepace = false;
    
    // cat '/tmp/rat/f   49' '/tmp/rat/f   74' '/tmp/rat/f   62'

    for c in cleaned.chars(){
        // println!("{:?}", new_input);
        // println!("{:?}", commands);
        // print!("------------\n");
        if c == '\''{
            if is_quoted{
                is_quoted = false;
                let mut word: String = new_input.into_iter().collect();
                word = word.trim().to_string();
                commands.push(word);
                new_input = vec![];
                continue
            }else{
                is_quoted = true;
                continue
            }
        }
        
        if c.is_ascii_whitespace(){
            if is_quoted{
                new_input.push(c);
                continue
            }else{
                if !whitepace{
                    whitepace = true;
                    let mut word: String = new_input.into_iter().collect();
                    word = word.trim().to_string();
                    commands.push(word);
                    new_input = vec![];
                    continue
                }
                continue
            }
        }
        whitepace = false;
        new_input.push(c);

    }
    if new_input.len() > 0{
        let mut word: String = new_input.clone().into_iter().collect();
        word = word.trim().to_string();
        commands.push(word);
    }
    // println!("{:?}", new_input);
    // println!("{:?}", commands);

    // let cleaned: String = new_input.into_iter().collect();
    // let arguments: Vec<&str> = cleaned.split(" ").collect();
    // arguments.into_iter().map(|x| x.to_string()).collect()
    let filtered: Vec<String> = commands.into_iter().filter(|x| x != "").collect();
    // println!("{:?}", filtered);
    filtered
}

fn main() {
    let mut shell_command = ShellCommand::new();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let command = parse_arguments(&input);
        match command[0].as_str() {
            "exit" => break,
            "pwd" => shell_command.pwd(),
            "echo" => shell_command.echo(command[1..].to_vec()),
            "type" => shell_command.type_(&command[1]).unwrap(),
            "cd" => shell_command.cd(&command[1]).unwrap(),
            _ => shell_command.execute(&command[0], command[1..].to_vec()).unwrap()
        }
    }
}
