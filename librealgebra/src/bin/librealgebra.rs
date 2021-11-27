use la_parse::Logos;
use la_parse::Scope;
use la_parse::Token;
use la_parse::parse_term;
use la_simplify::Constants;
use la_simplify::Context;
use la_simplify::Session;
use la_simplify::Warner;
use la_simplify::builtins::Builtins;
use la_simplify::simplify;
use la_term::symbol::Symbols;
use std::io::Read;
use std::io::stdin;

fn main()
{
    let stdin = stdin();
    let mut stdin = stdin.lock();
    let mut input = String::new();
    stdin.read_to_string(&mut input).unwrap();

    let session = Session::new();
    let symbols = Symbols::new();
    let constants = Constants::new(&symbols);
    let builtins = Builtins::new(&constants);
    let warner = StderrWarner;

    let scope = Scope::new(None, []);
    let mut lexer = Token::lexer(&input).peekable();
    let term = parse_term(&symbols, &scope, &mut lexer).unwrap();

    let context = Context{
        recursion_limit: 16,
        builtins: &builtins,
        constants: &constants,
        session: &session,
        symbols: &symbols,
        warner: &warner,
    };

    let term = simplify(context, term);
    println!("{:#?}", term);
}

struct StderrWarner;

impl Warner for StderrWarner
{
}
