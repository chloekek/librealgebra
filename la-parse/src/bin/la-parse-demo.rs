use std::io::Read;
use std::io::stdin;

use la_parse::Scope;
use la_parse::Token;
use la_parse::parse_term;
use logos::Logos;

fn main()
{
    let stdin = stdin();
    let mut stdin = stdin.lock();
    let mut input = String::new();
    stdin.read_to_string(&mut input).unwrap();

    let scope = Scope::new(None, []);
    let mut lexer = Token::lexer(&input).peekable();
    let term = parse_term(&scope, &mut lexer);
    println!("{:#?}", term);
}
