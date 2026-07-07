#[allow(unused_imports)]
use parser::{Redirection, extract_redirection, parse_arguments};
use rustyline::history::{DefaultHistory, History};
use rustyline::{Cmd, CompletionType, Config, Editor, KeyEvent, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::{env, fs};
use unescape::unescape;
use jobs::{Jobs, Job};
use crate::declare::DeclaredVariables;
use crate::program_completion::ProgrammableCompletor;

pub mod fs_utils;
pub mod helper;
pub mod output;
pub mod parser;
pub mod jobs;
pub mod program_completion;
pub mod declare;

const BUILTIN_COMMANDS: [&'static str; 9] = ["echo", "exit", "type", "pwd", "cd", "history", "jobs", "complete", "declare"];

struct ShellCommand {
    pub fs_utils: fs_utils::FSUtils,
    output: output::Output,
    pub redirection: parser::Redirection,
    jobs_list: Jobs,
    declared_variables: DeclaredVariables
}

impl ShellCommand {
    pub fn new() -> Self {
        let utils = fs_utils::FSUtils::new();
        let output = output::Output::new();
        let jobs = Jobs::new();
        ShellCommand {
            fs_utils: utils,
            output,
            redirection: parser::Redirection::Standart,
            jobs_list: jobs,
            declared_variables: DeclaredVariables::new()
        }
    }

    pub fn echo(&self, input: Vec<String>) -> (Option<String>, Option<String>) {
        if input[0] == "-e" {
            let str_ = input[1..]
                .iter()
                .filter_map(|s| unescape(s)) // returns Option<String>
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

    pub fn execute_in_background(
        &mut self,
        input: &String,
        arguments: Vec<String>,
    ) -> (Option<String>, Option<String>){
        let executable_file_name = self.fs_utils.is_executable(&input);
        if let Some(_) = executable_file_name {
            let arg = arguments.clone();
            let output = Command::new(input)
                .args(arguments)
                .spawn()
                .unwrap();
            let command_id = output.id();
            let number = self.jobs_list.get_next_job_number();
            let result = format!("[{}] {}\n", number, command_id);
            self.jobs_list.add_job(
                Job::new(output, number, input.into(), &arg)
            );
            return (Some(result), None);
        }
        (None, Some(format!("{}: command not found\n", input)))
    }

    pub fn execute_in_pipeline(
        &self,
        input: &String,
        arguments: Vec<String>,
        stdin: Option<Stdio>,
        stdout: Stdio,
    ) -> Option<Child> {
        let executable_file_name = self.fs_utils.is_executable(&input);
        let stderr = Stdio::piped();
        if let Some(_) = executable_file_name {
            if let Some(prev_out) = stdin {
                let output = Command::new(input)
                    .args(arguments)
                    .stdin(prev_out)
                    .stdout(stdout)
                    .stderr(stderr)
                    .spawn()
                    .unwrap();
                Some(output)
            } else {
                let output = Command::new(input)
                    .args(arguments)
                    .stdout(stdout)
                    .stderr(stderr)
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

    pub fn history(
        &mut self,
        args: Vec<String>,
        rl_history: &mut DefaultHistory,
    ) -> (Option<String>, Option<String>) {
        let flag: Option<&str> = if let Some(flag) = args.first() {
            Some(flag.as_str())
        } else {
            None
        };
        match (flag, args.get(1)) {
            (Some("-r"), Some(path)) => {
                if prepend_history_header(Path::new(path)).is_err() {
                    return (
                        None,
                        Some(format!(
                            "history: cannot read history file '{}': No such file or directory\n",
                            path
                        )),
                    );
                }
                let load_result = rl_history.load(Path::new(path));
                strip_history_header(Path::new(path)).ok();
                if load_result.is_err() {
                    return (
                        None,
                        Some(format!(
                            "history: cannot read history file '{}': No such file or directory\n",
                            path
                        )),
                    );
                }
                (None, None)
            }
            (Some("-w"), Some(path)) => {
                rl_history.save(Path::new(path)).unwrap();
                strip_history_header(Path::new(path)).unwrap();
                (None, None)
            }
            (Some("-a"), Some(path)) => {
                rl_history.append(Path::new(&path)).unwrap();
                strip_history_header(Path::new(path)).unwrap();
                (None, None)
            }
            _ => {
                let mut history: Vec<(usize, &String)> = vec![];
                let history_indexed: Vec<(usize, &String)> = rl_history.into_iter().enumerate().collect();
                if let Some(number_of_commands) = flag{
                    let number_to_skip = number_of_commands.parse::<usize>();
                    if number_to_skip.is_ok(){
                        let skip = history_indexed.len().saturating_sub(number_to_skip.unwrap());
                        history = history_indexed.into_iter().skip(skip).collect();
                    }
                }else{
                    history = history_indexed.into_iter().collect();
                }

                let mut result = String::new();
                for (i, command) in history.iter() {
                    result.push_str(&format!("\t{} {}\n", i + 1, command));
                }
                (Some(result), None)
            }
        }
    }

    pub fn exit(&self, rl_history: &mut DefaultHistory){
        let history_file = env::var("HISTFILE");
        if let Some(path) = history_file.ok() {
            rl_history.append(Path::new(&path)).unwrap();
            strip_history_header(Path::new(&path)).unwrap();
        }
    }

    pub fn jobs(&mut self) -> (Option<String>, Option<String>) {
        let all_jobs = self.jobs_list.get_all_jobs(false);
        if all_jobs.is_empty() {
            return (None, None);
        }

        let str = all_jobs.join("\n") + "\n";
        (Some(str), None)
    }

    pub fn complete(&self, arguments: Vec<String>, prog_completer: &mut ProgrammableCompletor) -> (Option<String>, Option<String>){
        if arguments.len() < 2{
            return (None, Some(String::from("No command specified\n")));
        }
        if arguments[0] == "-p"{
            let completion = prog_completer.get_completion_info(&arguments[1]);
            if let Some(comp) = completion{
                let res = format!("complete -C '{}' {}\n", comp, &arguments[1]);
                return (Some(res), None);
            }
            return (None, Some(String::from(format!("complete: {}: no completion specification\n", arguments[1]))));
        }
        if arguments[0] == "-C"{
            prog_completer.register_completion(arguments[2].clone(), arguments[1].clone());
        }
        if arguments[0] == "-r"{
            prog_completer.remove_completion(arguments[1].clone());
        }

        (None, None)
    }

    fn declare(&mut self, arguments: &Vec<String>) -> (Option<String>, Option<String>) {
        if arguments.len() < 2{
            return (None, Some(String::from("No command specified\n")));
        }

        if arguments[1] == "-p"{
            let command = arguments[2].clone();
            let description = self.declared_variables.get(arguments[2].clone());
            return if let Some(description) = description {
                (Some(
                    String::from(format!("declare -- {}=\"{}\"\n", command, description))
                ), None)
            } else {
                (None, Some(String::from(format!("declare: {}: not found\n", command))))
            }
        }

        if arguments[1].contains("="){
            if let Some((comm, description)) = arguments[1].split_once("="){
                self.declared_variables.register(comm.to_string(), description.to_string());
            }
        }

        (None, None)

    }

    pub fn run_command(
        &mut self,
        commands: Vec<Vec<String>>,
        rl_editor: &mut Editor<helper::CommandLineHelper, DefaultHistory>
    ) -> Result<()> {
        let mut iter = commands.iter().peekable();
        let mut prev_prc: Option<Stdio> = None;
        let mut children: Vec<Child> = Vec::new();

        while let Some(comm) = iter.next() {
            let (mut command_and_args, redirection) = extract_redirection(&comm);
            let mut command = command_and_args[0].as_str();
            if command_and_args.last().unwrap().as_str() == "&" {
                command_and_args.pop();
                command = "execute_background";
            }
            rl_editor.add_history_entry(&command_and_args.join(" "))?;
            self.redirection = redirection;
            self.output.sdtout(&String::new(), &self.redirection);
            self.output.stderr(&String::new(), &self.redirection);

            let (stdout, stderr): (Option<String>, Option<String>) = match command {
                "exit" => {
                    self.exit(rl_editor.history_mut());
                    std::process::exit(0)
                },
                "pwd" => self.pwd(),
                "echo" => self.echo(command_and_args[1..].to_vec()),
                "type" => self.type_(&command_and_args[1]),
                "cd" => self.cd(&command_and_args[1]),
                "history" => self.history(command_and_args[1..].to_vec(), rl_editor.history_mut()),
                "jobs" => self.jobs(),
                "complete" => {
                    let helper = rl_editor.helper_mut().unwrap();
                    self.complete(command_and_args[1..].to_vec(), &mut helper.programmable_completor)
                },
                "declare" => self.declare(&command_and_args),
                "execute_background" => self.execute_in_background(&command_and_args[0], command_and_args[1..].to_vec()),
                _ => {
                    if iter.peek().is_some() {
                        let result = self.execute_in_pipeline(
                            &command_and_args[0],
                            command_and_args[1..].to_vec(),
                            prev_prc.take(),
                            Stdio::piped(),
                        );
                        if result.is_none() {
                            (None, Some(format!("{}: command not found\n", command_and_args[0])))
                        } else {
                            let mut child = result.unwrap();
                            prev_prc = Some(Stdio::from(child.stdout.take().unwrap()));
                            children.push(child);
                            (None, None)
                        }
                    } else {
                        let stdout_stdio = match &self.redirection {
                            Redirection::RedirectStdout(path) => {
                                Stdio::from(fs::File::create(path).unwrap())
                            }
                            Redirection::AppendStdout(path) => Stdio::from(
                                fs::File::options()
                                    .append(true)
                                    .create(true)
                                    .open(path)
                                    .unwrap(),
                            ),
                            _ => Stdio::inherit(),
                        };
                        let result = self.execute_in_pipeline(
                            &command_and_args[0],
                            command_and_args[1..].to_vec(),
                            prev_prc.take(),
                            stdout_stdio,
                        );
                        let mut stdout: Option<String> = None;
                        let mut stderr: Option<String> = None;
                        if result.is_none() {
                            (None, Some(format!("{}: command not found\n", command_and_args[0])))
                        } else {
                            if let Some(child) = result {
                                let output = child.wait_with_output()?;
                                stdout = Some(String::from_utf8_lossy(&output.stdout).to_string());
                                stderr = Some(String::from_utf8_lossy(&output.stderr).to_string());
                            }
                            for child in &mut children {
                                let _ = child.kill();
                                let _ = child.wait();
                            }
                            (stdout, stderr)
                        }
                    }
                }
            };
            if stdout.is_some() {
                if iter.peek().is_some() {
                    let (reader, mut writer) = std::io::pipe()?;
                    writer.write_all(stdout.unwrap().as_bytes())?;
                    drop(writer);
                    prev_prc = Some(Stdio::from(reader));
                } else {
                    self.output.sdtout(&stdout.unwrap(), &self.redirection);
                }
            }

            if stderr.is_some() {
                self.output.stderr(&stderr.unwrap(), &self.redirection);
            }
        }

        Ok(())
    }
}

// Rustyline always adds header to the file
// By assignment, the file has to contain only commands
// Remove header after write to the file and add before load from it
fn strip_history_header(path: &Path) -> std::io::Result<()> {
    let content = fs::read_to_string(path)?;
    let stripped = content.strip_prefix("#V2\n").unwrap_or(&content);
    fs::write(path, stripped)
}

fn prepend_history_header(path: &Path) -> std::io::Result<()> {
    let content = fs::read_to_string(path)?;
    fs::write(path, format!("#V2\n{}", content))
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
    rl.bind_sequence(KeyEvent::alt('n'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);

    let history_file = env::var("HISTFILE");
    if let Some(path) = history_file.ok() {
        prepend_history_header(Path::new(&path))?;
        rl.load_history(Path::new(&path))?;
        strip_history_header(Path::new(&path))?;
    }

    loop {
        let jobs = shell_command.jobs_list.get_all_jobs(true);
        if !jobs.is_empty() {
            shell_command.output.sdtout(&format!("{}\n", jobs.join("\n")), &Redirection::Standart);
        }
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
        shell_command.run_command(commands, &mut rl)?;
    }
}
