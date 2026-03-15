#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::Path;
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

const BUILTIN_COMMANDS: [&'static str; 3] = ["echo", "exit", "type"];

fn is_executable(input: &str) -> Option<String> {
    let key = "PATH";
    match env::var_os(key) {
        Some(paths) => {
            for path in env::split_paths(&paths) {
                let joined = path.join(input);
                let file_name = joined.to_str().unwrap();

                if Path::new(file_name).exists() {
                    if let Some(executable) = std::fs::metadata(file_name).ok().map(|m| m.permissions().mode() & 0o111 != 0){
                        if executable{
                            return Some(file_name.to_string());
                        }
                    }

                }
            }
        },
        None => {
            println!("{key} is not defined in the environment.");
            return None;
        }
    }

    None
}

fn type_command(input: &str) -> io::Result<()> {
    if BUILTIN_COMMANDS.contains(&input) {
        println!("{} is a shell builtin", input);
        return Ok(());
    };

    let executable_file_name = is_executable(&input);
    if let Some(file_name) = executable_file_name {
        println!("{} is {}", input, file_name);
    }else{
        println!("{} not found", input);
    }
    Ok(())
}

fn execute_command(input: &str, arguments: Vec<&str>) -> io::Result<()> {
    let executable_file_name = is_executable(&input);
    if let Some(file_name) = executable_file_name {
        let output = Command::new(file_name).args(arguments).output()?;
        if output.status.success(){
            io::stdout().write_all(&output.stdout)?;
        }else{
            io::stderr().write_all(&output.stderr)?;
        }
    }else{
        println!("{}: command not found", input);
    }
    Ok(())
}

fn parse_arguments(input: &String) -> Vec<&str> {
    let arguments: Vec<&str> = input.split(" ").collect();
    arguments
}

fn main() {

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        if input == "exit"{
            break;
        }else if input.starts_with("echo"){
            println!("{}", &input[5..]);
        }else if input.starts_with("type"){
            type_command(&input[5..]).unwrap();
        }else{
            let command = parse_arguments(&input);
            execute_command(command[0], command[1..].to_vec()).unwrap();
        }
    }
}
