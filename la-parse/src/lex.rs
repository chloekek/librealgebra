use alloc::vec::Vec;
use logos::Lexer;
use logos::Logos;
use num_bigint::BigUint;
use num_traits::Num;

/// Token generated during lexing.
#[derive(Debug, Logos)]
pub enum Token
{
    /// Emitted when there is a lexical error.
    #[error]
    #[regex(r"[\t\n\v\f\r ]+", logos::skip)]
    Error,

    /// `,`.
    #[token(",")]
    Comma,

    /// `|`.
    #[token("|")]
    Pipe,

    /// `~`.
    #[token("~")]
    Tilde,

    /// `(`.
    #[token("(")]
    LeftParenthesis,

    /// `)`.
    #[token(")")]
    RightParenthesis,

    /// Integer token.
    #[regex(r"[0-9]+", lex_integer)]
    Integer(BigUint),

    /// String token.
    #[regex(r#""[^"]+""#, lex_string)]
    String(Vec<u8>),

    /// Identifier token.
    ///
    /// Whether this is interpreted as a symbol or as a variable
    /// depends on the scope given to the parser; see [`Scope`].
    ///
    /// [`Scope`]: `crate::Scope`
    #[regex(r"[A-Za-z]+", lex_identifier)]
    Identifier(Vec<u8>),
}

fn lex_integer(lex: &mut Lexer<Token>) -> Option<BigUint>
{
    BigUint::from_str_radix(lex.slice(), 10).ok()
}

fn lex_string(lex: &mut Lexer<Token>) -> Vec<u8>
{
    let input = lex.slice();
    input[1 .. input.len() - 1].into()
}

fn lex_identifier(lex: &mut Lexer<Token>) -> Vec<u8>
{
    lex.slice().into()
}

#[cfg(test)]
mod tests
{
    use super::*;

    use alloc::format;

    #[test]
    fn fine()
    {
        let mut lex = Token::lexer(r#",|~()123"Abc"Abc"#);
        let mut next = || format!("{:?}", lex.next());
        assert_eq!(next(), "Some(Comma)");
        assert_eq!(next(), "Some(Pipe)");
        assert_eq!(next(), "Some(Tilde)");
        assert_eq!(next(), "Some(LeftParenthesis)");
        assert_eq!(next(), "Some(RightParenthesis)");
        assert_eq!(next(), "Some(Integer(123))");
        assert_eq!(next(), "Some(String([65, 98, 99]))");
        assert_eq!(next(), "Some(Identifier([65, 98, 99]))");
        assert_eq!(next(), "None");
    }
}
