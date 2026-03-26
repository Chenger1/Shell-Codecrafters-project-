use std::vec;

const BUILTIN_COMMANDS: [&'static str; 6] = [">", "1>", "2>", ">>", "1>>", "2>>"];

pub enum Redirection{
    Standart,
    RedirectStdout(String),
    RedirectStdErr(String),
    AppendStdout(String),
    AppendStderr(String)
}

impl Redirection{
    pub fn math_redirection(symbol: &String, path: String) -> Redirection{
        match symbol.as_str(){
            ">" | "1>" => Redirection::RedirectStdout(path),
            "2>" => Redirection::RedirectStdErr(path),
            ">>" | "1>>" => Redirection::AppendStdout(path),
            "2>>" => Redirection::AppendStderr(path),
            _ => Redirection::Standart
        }
    }
    
    pub fn destination_path(&self) -> Option<String>{
        match self{
            Redirection::RedirectStdout(path) => return Some(path.clone()),
            _ => None
        }
    }
}

enum State{
    Whitespace,
    Unquoted,
    UnquotedBackSlash,
    SingleQuoted,
    DoubleQuoted,
    DoubleQuotedBackslash
}

pub fn parse_arguments(input: &str) -> Vec<String>{
    let mut tokens: Vec<String> = vec![];
    let mut current = String::new();
    let mut state = State::Whitespace;

    for c in input.trim().chars(){
        state = match (state, c){
            (State::Whitespace, '\'') => State::SingleQuoted,
            (State::Whitespace, '"') => State::DoubleQuoted,
            (State::Whitespace, '\\') => State::UnquotedBackSlash,
            (State::Whitespace, c) if c.is_ascii_whitespace() => State::Whitespace,
            (State::Whitespace, c) => {
                current.push(c);
                State::Unquoted
            },
            (State::Unquoted, c) if c.is_ascii_whitespace() => {
                tokens.push(current);
                current = String::new();
                State::Whitespace
            },
            (State::Unquoted, '\'') => State::SingleQuoted,
            (State::Unquoted, '"') => State::DoubleQuoted,
            (State::Unquoted, '\\') => State::UnquotedBackSlash,
            (State::Unquoted, c) => {
                current.push(c);
                State::Unquoted
            },
            (State::UnquotedBackSlash, c) => { 
                current.push(c); 
                State::Unquoted 
            },
            (State::SingleQuoted, '\'') => State::Unquoted,
            (State::SingleQuoted, c)    => { 
                current.push(c); 
                State::SingleQuoted 
            },
            (State::DoubleQuoted, '"')  => State::Unquoted,
            (State::DoubleQuoted, '\\') => State::DoubleQuotedBackslash,
            (State::DoubleQuoted, c)    => { 
                current.push(c); 
                State::DoubleQuoted 
            },
            (State::DoubleQuotedBackslash, c @ ('"' | '\\' | '$' | '`' | '\n')) => {
                current.push(c);
                State::DoubleQuoted
            },
            (State::DoubleQuotedBackslash, c) => {
                current.push('\\'); 
                current.push(c); State::DoubleQuoted
            }
        }   
    }

    if !(matches!(state, State::Whitespace)){
        tokens.push(current);
    }
    tokens
}

pub fn extract_redirection(arguments: &Vec<String>) -> (Vec<String>, Redirection){
    let redirection_symbol = arguments.iter().find(|item| BUILTIN_COMMANDS.contains(&item.as_str()));
    if let Some(red_symbol) = redirection_symbol{
        let parts: Vec<&[String]> = arguments.splitn(2, |item| item == red_symbol).collect();
        let (command, redirect_path) = (&parts[0], &parts[1]);
        return (command.to_vec(), Redirection::math_redirection(red_symbol, redirect_path.join("")))
    }else{
        return (arguments.to_vec(), Redirection::Standart)
    }
}


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_single_quotes(){
        assert_eq!(parse_arguments("echo 'hello    world'"), vec!["echo", "hello    world"]);
        assert_eq!(parse_arguments("echo hello    world"), vec!["echo", "hello", "world"]);
        assert_eq!(parse_arguments("echo 'hello''world'"), vec!["echo", "helloworld"]);
        assert_eq!(parse_arguments("echo hello''world"), vec!["echo", "helloworld"]);
    }

    #[test]
    fn test_double_quotes(){
        assert_eq!(parse_arguments("echo \"hello    world\""), vec!["echo", "hello    world"]);
        assert_eq!(parse_arguments("echo \"hello\"\"world\""), vec!["echo", "helloworld"]);
        assert_eq!(parse_arguments("echo \"hello\" \"world\""), vec!["echo", "hello", "world"]);
        assert_eq!(parse_arguments("echo \"shell's test\""), vec!["echo", "shell's test"]);
    }

    #[test]
    fn test_backslash(){
        assert_eq!(parse_arguments(r"echo three\ \ \ spaces"), vec!["echo", "three   spaces"]);
        assert_eq!(parse_arguments(r"echo before\     after"), vec!["echo", "before ", "after"]);
        assert_eq!(parse_arguments(r"echo test\nexample"), vec!["echo", "testnexample"]);
        assert_eq!(parse_arguments(r"echo hello\\world"), vec!["echo", "hello\\world"]);
        assert_eq!(parse_arguments(r"echo \'hello\'"), vec!["echo", "'hello'"]); 
    }

    #[test]
    fn test_backslash_single_quotes(){
        assert_eq!(parse_arguments(r"echo 'shell\\\nscript'"), vec!["echo", r"shell\\\nscript"]);
        assert_eq!(parse_arguments(r#"echo 'example\"test'"#), vec!["echo", r#"example\"test"#]);
        assert_eq!(parse_arguments(r"echo 'multiple\\slashes'"), vec!["echo", r"multiple\\slashes"]);
        assert_eq!(parse_arguments(r#"echo 'every\"thing_is\"literal'"#), vec!["echo", r#"every\"thing_is\"literal"#]);
    }

    #[test]
    fn test_backslash_double_quotes(){
        assert_eq!(parse_arguments(r#"echo "just'one'\\n'backslash""#), vec!["echo", r#"just'one'\n'backslash"#]);
        assert_eq!(parse_arguments(r#"echo "inside\"literal_quote."outside\"#), vec!["echo", r#"inside"literal_quote.outside"#]);
    }

    #[test]
    fn test_redirection(){
        let arguments = parse_arguments("echo hello > output.txt");
        let (command_arguments, redirection) = extract_redirection(&arguments);
        assert_eq!(command_arguments, vec!["echo", "hello"]);
        assert!(matches!(redirection, Redirection::RedirectStdout(ref path) if path == "output.txt"));
    }
}