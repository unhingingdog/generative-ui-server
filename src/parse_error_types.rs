#[derive(Debug, PartialEq)]
pub enum JSONParseError {
    QuoteCharAfterKeyClose,
    QuoteCharAfterValueClose,
    QuoteCharInNonStringData,
    UnexpectedCharInNonStringData,
    UnexpectedEscape,
    UnexpectedComma,
    UnexpectedColon,
    TokenParseErrorMisc(&'static str),
}
