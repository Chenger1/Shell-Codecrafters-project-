#[allow(unused_imports)]
use std::io::{self, Write};

const BUILTIN_COMMANDS: [&'static str; 3] = ["echo", "exit", "type";

fn type_command(input: &String){
    if BUILTIN_COMMANDS.contains(&input.as_str()) {
        println!("{} is a shell builtin", input);
    }else{
        println!("{} not found", input);
    }
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
            type_command(&input);
        }else{
            println!("{}: command not found", input);
        }
    }
}
