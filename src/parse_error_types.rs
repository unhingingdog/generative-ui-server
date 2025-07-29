#[derive(Debug, PartialEq)]
pub enum JSONParseError {
    QuoteCharAfterKeyClose,
    QuoteCharAfterValueClose,
    QuoteCharInNonStringData,
    UnexpectedCharInNonStringData,
    UnexpectedEscape,
    UnexpectedComma,
    UnexpectedColon,
    UnexpectedOpenBrace,
    UnexpectedCloseBrace,
    TokenParseErrorMisc(&'static str),
}
