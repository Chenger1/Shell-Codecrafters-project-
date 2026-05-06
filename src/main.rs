use parser::{extract_redirection, parse_arguments, Redirection};
use rustyline::{CompletionType, Config, Editor, Result};
#[allow(unused_imports)]
use std::io::{self, Write, pipe};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::{env, fs};
use unescape::unescape;

pub mod fs_utils;
pub mod helper;
pub mod output;
pub mod parser;

const BUILTIN_COMMANDS: [&'static str; 5] = ["echo", "exit", "type", "pwd", "cd"];

struct ShellCommand {
    pub fs_utils: fs_utils::FSUtils,
    output: output::Output,
    pub redirection: parser::Redirection,
}

impl ShellCommand {
    pub fn new() -> Self {
        let utils = fs_utils::FSUtils::new();
        let output = output::Output::new();
        ShellCommand {
            fs_utils: utils,
            output: output,
            redirection: parser::Redirection::Standart,
        }
    }

    pub fn echo(&self, input: Vec<String>) -> (Option<String>, Option<String>) {
        if input[0] == "-e"{
            let str_ = input[1..].iter()
                .filter_map(|s| unescape(s))  // returns Option<String>
                .collect::<Vec<_>>()
                .join(" ");
            let result = format!("{}\n", str_);
            return (Some(result), None);
        }
        let str_ = input.join(" ");
        let result = format!("{}\n", str_);
        (Some(result), None)
    }

    pub fn pwd(&self) -> (Option<String>, Option<String>) {
        let path = fs::canonicalize(".").unwrap();
        let result = format!("{}\n", path.to_str().unwrap());
        (Some(result), None)
    }

    pub fn execute(
        &self,
        input: &String,
        arguments: Vec<String>,
    ) -> (Option<String>, Option<String>) {
        let executable_file_name = self.fs_utils.is_executable(&input);
        if let Some(_) = executable_file_name {
            let output = Command::new(input)
                .args(arguments)
                .output()
                .unwrap();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            (Some(stdout), Some(stderr))
        } else {
            let result = format!("{}: command not found\n", input);
            (None, Some(result))
        }
    }

    pub fn execute_in_pipeline(
        &self,
        input: &String,
        arguments: Vec<String>,
        stdin: Option<Stdio>,
        stdout: Stdio,
    ) -> Option<Child> {
        let executable_file_name = self.fs_utils.is_executable(&input);
        if let Some(_) = executable_file_name {
            if let Some(prev_out) = stdin {
                let output = Command::new(input)
                    .args(arguments)
                    .stdin(prev_out)
                    .stdout(stdout)
                    .spawn()
                    .unwrap();
                Some(output)
            } else {
                let output = Command::new(input)
                    .args(arguments)
                    .stdout(stdout)
                    .spawn()
                    .unwrap();
                Some(output)
            }
        } else {
            None
        }
    }

    pub fn type_(&self, input: &String) -> (Option<String>, Option<String>) {
        if BUILTIN_COMMANDS.contains(&input.as_str()) {
            let result = format!("{} is a shell builtin\n", input);
            return (Some(result), None);
        };

        let executable_file_name = self.fs_utils.is_executable(&input);
        if let Some(file_name) = executable_file_name {
            let result = format!("{} is {}\n", input, file_name);
            return (Some(result), None);
        } else {
            let result = format!("{} not found\n", input);
            return (None, Some(result));
        }
    }

    pub fn cd(&mut self, path: &String) -> (Option<String>, Option<String>) {
        let mut desired_path = path.to_string();
        if path == "~" {
            let home = env::var("HOME").unwrap();
            desired_path = home;
        }

        let is_absolute = Path::new(&desired_path).is_absolute();
        if !self.fs_utils.is_exist(&desired_path, is_absolute) {
            let result = format!("cd: {}: No such file or directory\n", desired_path);
            return (None, Some(result));
        }

        let new_dir = Path::new(&desired_path);
        env::set_current_dir(&new_dir).unwrap();
        (None, None)
    }

    pub fn run_single_command(&mut self, comm: Vec<String>) -> Result<()> {
        let (command, redirection) = extract_redirection(&comm);
        self.redirection = redirection;
        self.output.sdtout(&String::new(), &self.redirection);
        self.output.stderr(&String::new(), &self.redirection);
        let (stdout, stderr): (Option<String>, Option<String>) = match command[0].as_str() {
            "exit" => std::process::exit(0),
            "pwd" => self.pwd(),
            "echo" => self.echo(command[1..].to_vec()),
            "type" => self.type_(&command[1]),
            "cd" => self.cd(&command[1]),
            _ => self.execute(&command[0], command[1..].to_vec()),
        };
        if stdout.is_some() {
            self.output.sdtout(&stdout.unwrap(), &self.redirection);
        }

        if stderr.is_some() {
            self.output.stderr(&stderr.unwrap(), &self.redirection);
        }

        Ok(())
    }

    pub fn run_pipeline(&mut self, commands: Vec<Vec<String>>) -> Result<()> {
        let mut iter = commands.iter().peekable();
        let mut prev_prc: Option<Stdio> = None;
        let mut children: Vec<Child> = Vec::new();

        while let Some(comm) = iter.next() {
            let (command, redirection) = extract_redirection(&comm);
            self.redirection = redirection;
            self.output.sdtout(&String::new(), &self.redirection);
            self.output.stderr(&String::new(), &self.redirection);

            let (stdout, stderr): (Option<String>, Option<String>) = match command[0].as_str() {
                "exit" => std::process::exit(0),
                "pwd" => self.pwd(),
                "echo" => self.echo(command[1..].to_vec()),
                "type" => self.type_(&command[1]),
                "cd" => self.cd(&command[1]),
                _ => {
                    if iter.peek().is_some() {
                        let result = self.execute_in_pipeline(
                            &command[0],
                            command[1..].to_vec(),
                            prev_prc.take(),
                            Stdio::piped(),
                        );
                        let mut child = result.unwrap();
                        prev_prc = Some(Stdio::from(child.stdout.take().unwrap()));
                        children.push(child);
                        (None, None)
                    } else {
                        let stdout_stdio = match &self.redirection {
                            Redirection::RedirectStdout(path) =>
                                Stdio::from(fs::File::create(path).unwrap()),
                            Redirection::AppendStdout(path) =>
                                Stdio::from(fs::File::options().append(true).create(true).open(path).unwrap()),
                            _ => Stdio::inherit(),
                        };
                        let result = self.execute_in_pipeline(
                            &command[0],
                            command[1..].to_vec(),
                            prev_prc.take(),
                            stdout_stdio,
                        );
                        if let Some(mut child) = result {
                            let _ = child.wait();
                        }
                        for child in &mut children {
                            let _ = child.kill();
                            let _ = child.wait();
                        }
                        (None, None)
                    }
                }
            };

            if stdout.is_some(){
                if iter.peek().is_some() {
                    let (reader, mut writer) = std::io::pipe()?;
                    writer.write_all(stdout.unwrap().as_bytes())?;
                    drop(writer);
                    prev_prc = Some(Stdio::from(reader));
                }else{
                    self.output.sdtout(&stdout.unwrap(), &self.redirection);
                }
            }

            if stderr.is_some(){
                self.output.stderr(&stderr.unwrap(), &self.redirection);
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let mut shell_command = ShellCommand::new();
    let config = Config::builder()
        .completion_show_all_if_ambiguous(true)
        .completion_type(CompletionType::List)
        .build();
    let mut rl = Editor::with_config(config)?;
    let path_executables = shell_command.fs_utils.get_path_executables();
    let helper = helper::CommandLineHelper::new(BUILTIN_COMMANDS, path_executables);
    rl.set_helper(Some(helper));

    loop {
        let input = rl.readline("$ ")?;
        if input.is_empty() {
            continue;
        }

        if let Some(h) = rl.helper_mut() {
            h.clear_state();
        }
        let commands = parse_arguments(&input);
        if commands.is_empty() {
            continue;
        }

        if commands.len() == 1 {
            shell_command.run_single_command(commands[0].clone())?;
        } else {
            shell_command.run_pipeline(commands)?;
        }
    }
}
