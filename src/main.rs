mod handle_escape;
mod non_string_data;
mod parse_brace;
mod parse_colon;
mod parse_comma;
mod parse_error_types;
mod parse_quote_char;
mod state_types;
mod string_data;
mod structure_type;

use handle_escape::handle_escape;
use non_string_data::{is_non_string_data, parse_non_string_data};
use parse_brace::parse_brace;
use parse_colon::parse_colon;
use parse_comma::parse_comma;
use parse_error_types::JSONParseError;
use parse_quote_char::parse_quote_char;
use state_types::{BraceState, BracketState, JSONState, PrimValue, StringState, Token};
use string_data::{is_string_data, parse_string_data};
use structure_type::RecursiveStructureType;

fn parse_char(c: char, current_state: &mut JSONState) -> Result<Token, JSONParseError> {
    match c {
        '\\' => handle_escape(current_state),
        '"' => parse_quote_char(current_state),
        c if is_string_data(current_state) => parse_string_data(current_state),
        c if is_non_string_data(c, current_state) => parse_non_string_data(current_state),
        ',' => parse_comma(current_state),
        ':' => parse_colon(current_state),
        '{' => parse_brace(RecursiveStructureType::Open, current_state),
        '}' => parse_brace(RecursiveStructureType::Close, current_state),
        ' ' | '\t' | '\n' | '\r' => Ok(Token::Whitespace),
        _ => Ok(Token::Comma),
    }
}

fn balance_json(json: &str) -> &str {
    let stack: Vec<Token> = vec![];
    // whenever we hit a close token, we make sure there's a corresponding open token
    // on the top of the stack, and we pop it off (and discard it).
    // After we've parsed all of json, we then take any remaining tokens on the stack, pop them all
    // off, if it's an opening token, we append a closing token to the json content, otherwise we
    // ignore it (or maybe there's some more we could do here for robustness, like throw away a
    // trailing comma). we then return the close json content

    todo!();
}
fn main() {}
