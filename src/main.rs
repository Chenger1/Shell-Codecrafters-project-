#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        match &input[..] {
            "exit" => break,
            "echo" => {
                let input = input.replace("echo ", "").replace("\n", "");
                println!("{}", input);
            },
            _ => println!("{}: command not found", input)
        }
    }
}
