use crate::Error;
use crate::Result;
use crate::Scope;
use crate::Token;

use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::iter::Peekable;
use la_term::Term;
use la_term::lambda::Parameter;
use la_term::lambda::Strictness;
use la_term::symbol::Symbols;

/// Stream of tokens generated from text.
pub type Lexer<'a> = Peekable<logos::Lexer<'a, Token<'a>>>;

////////////////////////////////////////////////////////////////////////////////
// Terms

/// Parse a term from a token stream.
pub fn parse_term(symbols: &Symbols, scope: &Scope, lex: &mut Lexer)
    -> Result<Term>
{
    parse_term_2(symbols, scope, lex)
}

fn parse_term_2(symbols: &Symbols, scope: &Scope, lex: &mut Lexer)
    -> Result<Term>
{
    let mut term = parse_term_1(symbols, scope, lex)?;
    while let Some(arguments) = parse_argument_list(symbols, scope, lex)? {
        term = Term::application(term, arguments)?;
    }
    Ok(term)
}

fn parse_term_1(symbols: &Symbols, scope: &Scope, lex: &mut Lexer)
    -> Result<Term>
{
    match lex.next() {

        Some(Token::Pipe) => {
            let parameters = parse_comma_matches!(
                lex,
                Token::Pipe,
                |lex| parse_parameter(symbols, lex),
            )?;
            let body = {
                let parameters = parameters.iter().map(|p| p.name.clone());
                let scope = Scope::new(Some(scope), parameters);
                parse_term(symbols, &scope, lex)?
            };
            Term::lambda(parameters.into(), body)
                .map_err(Error::from)
        },

        Some(Token::LeftParenthesis) => {
            let term = parse_term(symbols, scope, lex)?;
            parse_exact_matches!(lex, Token::RightParenthesis)?;
            Ok(term)
        },

        Some(Token::Integer(value)) =>
            Term::integer(value)
                .map_err(Error::from),

        Some(Token::String(value)) =>
            Term::string(value.iter().copied())
                .map_err(Error::from),

        Some(Token::Identifier(ref name)) => {
            let name = symbols.get(name)?;
            match scope.get(&name) {
                Some(de_bruijn) =>
                    Term::variable(de_bruijn)
                        .map_err(Error::from),
                None =>
                    Ok(Term::symbol(name)),
            }
        },

        _ => todo!(),

    }
}

////////////////////////////////////////////////////////////////////////////////
// Components

fn parse_identifier<'a>(lex: &mut Lexer<'a>) -> Result<Cow<'a, [u8]>>
{
    let token = lex.next();
    match token {
        Some(Token::Identifier(name)) => Ok(name),
        _ => todo!(),
    }
}

fn parse_argument_list(symbols: &Symbols, scope: &Scope, lex: &mut Lexer)
    -> Result<Option<Vec<Term>>>
{
    if parse_optional_matches!(lex, Token::LeftParenthesis) {
        let arguments = parse_comma_matches!(
            lex,
            Token::RightParenthesis,
            |lex| parse_term(symbols, scope, lex),
        )?;
        Ok(Some(arguments))
    } else {
        Ok(None)
    }
}

fn parse_parameter(symbols: &Symbols, lex: &mut Lexer) -> Result<Parameter>
{
    let strictness = parse_strictness(lex);
    let name = parse_identifier(lex)?;
    let name = symbols.get(&name)?;
    Ok(Parameter{strictness, name})
}

fn parse_strictness(lex: &mut Lexer) -> Strictness
{
    let tilde = parse_optional_matches!(lex, Token::Tilde);
    if tilde {
        Strictness::NonStrict
    } else {
        Strictness::Strict
    }
}

////////////////////////////////////////////////////////////////////////////////
// Combinators

/// If the next token matches the predicate, consume it and return `true`.
/// Otherwise leave it in the token stream and return `false`.
fn parse_optional<F>(lex: &mut Lexer, pred: F) -> bool
    where F: FnOnce(&Token) -> bool
{
    lex.next_if(pred).is_some()
}

/// Read the next token and assert that it matches the predicate.
fn parse_exact<F>(lex: &mut Lexer, pred: F) -> Result<()>
    where F: FnOnce(&Token) -> bool
{
    let token = lex.next();
    match token {
        Some(ref token) if pred(token) => Ok(()),
        _ => todo!(),
    }
}

/// Parse a comma-separated list terminated by the given terminator.
/// A trailing comma is permitted at the end of the list.
fn parse_comma<F, G, T>(
    lex: &mut Lexer,
    mut is_terminator: F,
    mut parse_element: G,
) -> Result<Vec<T>>
    where F: FnMut(&Token) -> bool
        , G: FnMut(&mut Lexer) -> Result<T>
{
    let mut elements = Vec::new();
    if parse_optional(lex, &mut is_terminator) {
        return Ok(elements);
    }
    loop {
        let element = parse_element(lex)?;
        elements.push(element);
        if parse_optional_matches!(lex, Token::Comma) {
            if parse_optional(lex, &mut is_terminator) {
                break;
            }
            continue;
        }
        if parse_optional(lex, is_terminator) {
            break;
        }
        todo!();
    }
    Ok(elements)
}

macro parse_optional_matches($lex:expr, $token:pat $(,)?)
{
    parse_optional($lex, |token| matches!(token, $token))
}

macro parse_exact_matches($lex:expr, $token:pat $(,)?)
{
    parse_exact($lex, |token| matches!(token, $token))
}

macro parse_comma_matches($lex:expr, $token:pat, $parse_element:expr $(,)?)
{
    parse_comma($lex, |token| matches!(token, $token), $parse_element)
}
