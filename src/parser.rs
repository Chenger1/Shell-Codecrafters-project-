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
}