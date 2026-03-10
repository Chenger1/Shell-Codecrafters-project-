#[allow(unused_imports)]
use std::io::{self, Write};
use std::fs;
use std::env;

const BUILTIN_COMMANDS: [&'static str; 3] = ["echo", "exit", "type"];

fn type_command(input: &str) -> io::Result<()> {
    if BUILTIN_COMMANDS.contains(&input) {
        println!("{} is a shell builtin", input);
        return Ok(());
    }
    let key = "PATH";
    match env::var_os(key) {
        Some(paths) => {
            for path in env::split_paths(&paths) {
                let joined = path.join(input);
                let file_name = joined.to_str().unwrap();
                println!("{}", file_name);
                if file_name == input {
                    println!("{} is {}", input, path.display());
                }
            }
        },
        None => {
            println!("{key} is not defined in the environment.");
            return Ok(());
        }
    }

    println!("{} not found", input);
    Ok(())
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
            type_command(&input[5..]);
        }else{
            println!("{}: command not found", input);
        }
    }
}
