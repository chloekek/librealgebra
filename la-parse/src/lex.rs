pub use logos::Logos;

use alloc::borrow::Cow;
use logos::Lexer;

/// Token generated during lexing.
#[derive(Debug, Logos)]
pub enum Token<'a>
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
    Integer(i32),

    /// String token.
    #[regex(r#""[^"]+""#, lex_string)]
    String(Cow<'a, [u8]>),

    /// Identifier token.
    ///
    /// Whether this is interpreted as a symbol or as a variable
    /// depends on the scope given to the parser; see [`Scope`].
    ///
    /// [`Scope`]: `crate::Scope`
    #[regex(r"[A-Za-z]+", lex_identifier)]
    Identifier(Cow<'a, [u8]>),
}

fn lex_integer<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Option<i32>
{
    i32::from_str_radix(lex.slice(), 10).ok()
}

fn lex_string<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Cow<'a, [u8]>
{
    let input = lex.slice();
    input[1 .. input.len() - 1].as_bytes().into()
}

fn lex_identifier<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Cow<'a, [u8]>
{
    lex.slice().as_bytes().into()
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
