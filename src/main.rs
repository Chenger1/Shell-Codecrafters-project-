use std::{env, fs};
#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::Command;
use std::path::Path;
use rustyline::{Editor, Result, Config};
use parser::{parse_arguments, extract_redirection};

pub mod fs_utils;
pub mod parser;
pub mod output;
pub mod helper;

const BUILTIN_COMMANDS: [&'static str; 5] = ["echo", "exit", "type", "pwd", "cd"];

struct ShellCommand {
    pub fs_utils: fs_utils::FSUtils,
    output: output::Output,
    pub redirection: parser::Redirection
}

impl ShellCommand {
    pub fn new() -> Self{
        let utils = fs_utils::FSUtils::new();
        let output = output::Output::new();
        ShellCommand { fs_utils: utils, output: output, redirection: parser::Redirection::Standart}
    }
    
    pub fn echo(&self, input: Vec<String>){
        let str_ = input.join(" ");
        let result = format!("{}\n", str_);
        self.output.sdtout(&result, &self.redirection);
    }

    pub fn pwd(&self){
        let path = fs::canonicalize(".").unwrap();
        let result = format!("{}\n", path.to_str().unwrap());
        self.output.sdtout(&result, &self.redirection);
    }

    pub fn execute(&self, input: &String, arguments: Vec<String>) -> io::Result<()>{
        let executable_file_name = self.fs_utils.is_executable(&input);
        if let Some(_) = executable_file_name {
            let output = Command::new(input).args(arguments).output()?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if !stdout.is_empty(){
                self.output.sdtout(&stdout, &self.redirection);
            }
            if !output.stderr.is_empty(){
                self.output.stderr(&stderr, &self.redirection);
            }
        } else {
            let result = format!("{}: command not found\n", input);
            self.output.sdtout(&result, &self.redirection);
        }
        Ok(())
    }

    pub fn type_(&self, input: &String) -> io::Result<()>{
        if BUILTIN_COMMANDS.contains(&input.as_str()) {
            let result = format!("{} is a shell builtin\n", input);
            self.output.sdtout(&result, &self.redirection);
            return Ok(());
        };
        
        let executable_file_name = self.fs_utils.is_executable(&input);
        if let Some(file_name) = executable_file_name {
            let result = format!("{} is {}\n", input, file_name);
            self.output.sdtout(&result, &self.redirection);
        } else {
            let result = format!("{} not found\n", input);
            self.output.sdtout(&result, &self.redirection);
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
            let result = format!("cd: {}: No such file or directory\n", desired_path);
            self.output.sdtout(&result, &self.redirection);
            return Ok(())
        }

        let new_dir = Path::new(&desired_path);
        env::set_current_dir(&new_dir)?;
        Ok(())
    }

}


fn main() -> Result<()>{
    let mut shell_command = ShellCommand::new();
    let config = Config::builder().build();
    let mut rl = Editor::with_config(config)?;
    let path_executables = shell_command.fs_utils.get_path_executables();
    rl.set_helper(Some(helper::CommandLineHelper::new(BUILTIN_COMMANDS, path_executables)));

    loop {
        let input = rl.readline("$ ")?;
        let command = parse_arguments(&input);
        let (command, redirection) = extract_redirection(&command);
        shell_command.redirection = redirection;
        shell_command.output.sdtout(&String::new(), &shell_command.redirection);
        shell_command.output.stderr(&String::new(), &shell_command.redirection);
        match command[0].as_str() {
            "exit" => break,
            "pwd" => shell_command.pwd(),
            "echo" => shell_command.echo(command[1..].to_vec()),
            "type" => shell_command.type_(&command[1]).unwrap(),
            "cd" => shell_command.cd(&command[1]).unwrap(),
            _ => shell_command.execute(&command[0], command[1..].to_vec()).unwrap()
        }
    }

    Ok(())
}
