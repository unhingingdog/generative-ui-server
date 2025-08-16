use crate::{lexer::escape::handle_escaped_char, JSONState};

use super::{
    brace::parse_brace,
    bracket::parse_bracket,
    colon::parse_colon,
    comma::parse_comma,
    escape::handle_escape,
    lexer_types::RecursiveStructureType,
    non_string_data::{is_non_string_data, parse_non_string_data},
    quote::parse_quote_char,
    string_data::{is_string_data, parse_string_data},
    JSONParseError, Token,
};

pub fn parse_char(c: char, st: &mut JSONState) -> Result<Token, JSONParseError> {
    // 0) If we’re currently in Escaped state, resolve it *before anything else*
    //    (even before handling `"` or `\`). This prevents `\"` from closing the string
    //    and ensures `\n` flips Escaped -> Open.
    use crate::parser::state_types::*;
    if matches!(
        st,
        JSONState::Brace(BraceState::InKey(StringState::Escaped))
            | JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Escaped)))
            | JSONState::Bracket(BracketState::InValue(PrimValue::String(
                StringState::Escaped
            )))
    ) {
        return handle_escaped_char(c, st);
    }

    // 1) string controls win when inside strings (but not Escaped — handled above)
    match c {
        '\\' => return handle_escape(st),
        '"' => return parse_quote_char(st),
        _ => {}
    }

    // 2) delimiters must preempt non-string parsing when value is completable
    let in_completable = matches!(
        st,
        JSONState::Brace(BraceState::InValue(
            PrimValue::String(StringState::Closed)
                | PrimValue::NonString(NonStringState::Completable(_))
                | PrimValue::NestedValueCompleted
        )) | JSONState::Bracket(BracketState::InValue(
            PrimValue::String(StringState::Closed)
                | PrimValue::NonString(NonStringState::Completable(_))
                | PrimValue::NestedValueCompleted
        ))
    );

    if in_completable {
        match c {
            ',' => return parse_comma(st),
            '}' => return parse_brace(RecursiveStructureType::Close, st),
            ']' => return parse_bracket(RecursiveStructureType::Close, st),
            _ => {}
        }
    }

    // 3) data lexers
    if is_string_data(st) {
        return parse_string_data(st);
    }
    if is_non_string_data(c, st) {
        return parse_non_string_data(c, st);
    }

    // 4) remaining structural / whitespace / error
    match c {
        '{' => parse_brace(RecursiveStructureType::Open, st),
        '}' => parse_brace(RecursiveStructureType::Close, st),
        '[' => parse_bracket(RecursiveStructureType::Open, st),
        ']' => parse_bracket(RecursiveStructureType::Close, st),
        ':' => parse_colon(st),
        ',' => parse_comma(st),
        ' ' | '\t' | '\n' | '\r' => Ok(Token::Whitespace),
        _ => Err(JSONParseError::InvalidCharEncountered),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::state_types::{BraceState, BracketState, PrimValue, StringState};

    use super::*;

    // Helper function to create a state for being inside an open string value.
    fn in_string_value_state() -> JSONState {
        JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Open)))
    }

    // Helper function to create a state where a structural token is expected.
    fn expecting_value_state() -> JSONState {
        JSONState::Brace(BraceState::ExpectingValue)
    }

    #[test]
    fn test_order_double_quote_is_prioritized_over_string_data() {
        // This test ensures that when inside a string, the `"` character is handled
        // by `parse_quote_char` to close the string, NOT by `parse_string_data`.
        let mut state = in_string_value_state();

        // `parse_quote_char` should be called and return `CloseStringData`.
        // If `parse_string_data` were called, it would return `OpenStringData`.
        let result = parse_char('"', &mut state);

        assert_eq!(result, Ok(Token::CloseStringData));
    }

    #[test]
    fn test_order_backslash_is_prioritized_over_string_data() {
        // This test ensures that when inside a string, the `\` character is handled
        // by `handle_escape`, not `parse_string_data`.
        let mut state = in_string_value_state();

        // `handle_escape` should be called, which transitions the state to `Escaped`.
        let result = parse_char('\\', &mut state);

        assert_eq!(result, Ok(Token::StringContent)); // `handle_escape` returns this
        assert_eq!(
            state,
            JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Escaped)))
        );
    }

    #[test]
    fn test_order_string_data_is_prioritized_over_structural_tokens() {
        // This test ensures that when inside a string, a character like `{` is
        // treated as string content, not as a structural token to open a new object.
        let mut state = in_string_value_state();
        let original_state = state.clone();

        // `parse_string_data` should be called, which just returns `OpenStringData`
        // and does not change the state.
        let result = parse_char('{', &mut state);

        assert_eq!(result, Ok(Token::StringContent));
        // The state should not have changed, proving `parse_brace` was not called.
        assert_eq!(state, original_state);
    }

    #[test]
    fn test_order_structural_token_is_handled_when_not_in_string() {
        // This test confirms that when NOT inside a string, a character like `{`
        // is correctly handled by `parse_brace`.
        let mut state = expecting_value_state();

        // `parse_brace` should be called, which changes the state to a new, empty object.
        let result = parse_char('{', &mut state);

        assert_eq!(result, Ok(Token::OpenBrace));
        assert_eq!(state, JSONState::Brace(BraceState::Empty));
    }

    #[test]
    fn test_order_invalid_char_falls_through_to_error() {
        // This test ensures that a character that is invalid in the current context
        // correctly falls through all other arms and returns the final error.
        let mut state = expecting_value_state();

        // The character '#' is not a valid start to a non-string value and is not
        // a structural token, so it should result in an error.
        let result = parse_char('#', &mut state);

        assert_eq!(result, Err(JSONParseError::InvalidCharEncountered));
    }

    #[test]
    fn test_order_whitespace_is_handled_correctly_between_tokens() {
        // This test verifies that whitespace is correctly identified as such and does
        // not incorrectly trigger other parsing arms, like the error arm.
        let mut state = JSONState::Brace(BraceState::ExpectingKey);
        let original_state = state.clone();

        let result = parse_char(' ', &mut state);

        assert_eq!(result, Ok(Token::Whitespace));
        // The state should be unchanged after parsing whitespace.
        assert_eq!(state, original_state);
    }

    #[test]
    fn delimiters_preempt_nonstring_in_object_completable_comma() {
        // { "a": 1 , ...
        let mut st = JSONState::Brace(BraceState::ExpectingValue);
        assert_eq!(parse_char('1', &mut st), Ok(Token::NonStringData)); // now completable
        let got = parse_char(',', &mut st);
        assert_eq!(got, Ok(Token::Comma));
        assert_eq!(st, JSONState::Brace(BraceState::ExpectingKey));
    }

    #[test]
    fn delimiters_preempt_nonstring_in_object_close_brace() {
        // { "a": 1 }
        let mut st = JSONState::Brace(BraceState::ExpectingValue);
        assert_eq!(parse_char('1', &mut st), Ok(Token::NonStringData)); // now completable
        let got = parse_char('}', &mut st);
        assert_eq!(got, Ok(Token::CloseBrace));
        // don’t assert exact state beyond token; upstream stack determines it
    }

    #[test]
    fn delimiters_preempt_nonstring_in_array_comma() {
        // [ 1 , ...
        let mut st = JSONState::Bracket(BracketState::ExpectingValue);
        assert_eq!(parse_char('1', &mut st), Ok(Token::NonStringData)); // now completable
        let got = parse_char(',', &mut st);
        assert_eq!(got, Ok(Token::Comma));
        assert_eq!(st, JSONState::Bracket(BracketState::ExpectingValue));
    }

    #[test]
    fn delimiters_preempt_nonstring_in_array_close_bracket() {
        // [ 1 ]
        let mut st = JSONState::Bracket(BracketState::ExpectingValue);
        assert_eq!(parse_char('1', &mut st), Ok(Token::NonStringData)); // now completable
        let got = parse_char(']', &mut st);
        assert_eq!(got, Ok(Token::CloseBracket));
    }

    #[test]
    fn delimiters_preempt_after_string_value_closed_in_object() {
        // { "a": "x" , ... }  — after closing quote, comma routes before data lexers
        let mut st = JSONState::Brace(BraceState::ExpectingValue);
        // open string
        assert_eq!(parse_char('"', &mut st), Ok(Token::OpenStringData));
        // some content
        assert_eq!(parse_char('x', &mut st), Ok(Token::StringContent));
        // close string
        assert_eq!(parse_char('"', &mut st), Ok(Token::CloseStringData));
        // comma should be handled by comma parser, moving to ExpectingKey
        let got = parse_char(',', &mut st);
        assert_eq!(got, Ok(Token::Comma));
        assert_eq!(st, JSONState::Brace(BraceState::ExpectingKey));
    }

    #[test]
    fn delimiters_preempt_after_string_value_closed_in_array() {
        // [ "x" , ... ]
        let mut st = JSONState::Bracket(BracketState::ExpectingValue);
        assert_eq!(parse_char('"', &mut st), Ok(Token::OpenStringData));
        assert_eq!(parse_char('x', &mut st), Ok(Token::StringContent));
        assert_eq!(parse_char('"', &mut st), Ok(Token::CloseStringData));
        let got = parse_char(',', &mut st);
        assert_eq!(got, Ok(Token::Comma));
        assert_eq!(st, JSONState::Bracket(BracketState::ExpectingValue));
    }

    #[test]
    fn escaped_state_consumes_regular_escape_char() {
        // start in Escaped state after seeing a backslash
        let mut st = JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Escaped)));

        // feeding 'n' is resolved by handle_escaped_char and returns StringContent
        let got = parse_char('n', &mut st);
        assert_eq!(got, Ok(Token::StringContent));

        // state should now be back to Open (normal string parsing)
        assert_eq!(
            st,
            JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Open)))
        );
    }

    #[test]
    fn escaped_state_with_unicode_u_stays_escaped_and_is_not_closable() {
        // start in Escaped state
        let mut st = JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Escaped)));

        let got = parse_char('u', &mut st);
        assert_eq!(got, Err(JSONParseError::NotClosableInsideUnicode));

        // state remains Escaped so caller knows we’re mid-unicode sequence
        assert_eq!(
            st,
            JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Escaped)))
        );
    }

    // delimiter check (`in_completable`) correctly handles the `NestedValueCompleted` state.
    #[test]
    fn delimiters_preempt_after_nested_value_completed() {
        // Simulates being in an array after a nested object has just closed: `[ { ... } ,`
        let mut st_array_comma =
            JSONState::Bracket(BracketState::InValue(PrimValue::NestedValueCompleted));
        let res_array_comma = parse_char(',', &mut st_array_comma);
        assert_eq!(res_array_comma, Ok(Token::Comma));
        assert_eq!(
            st_array_comma,
            JSONState::Bracket(BracketState::ExpectingValue)
        );

        // Simulates being in an array after a nested object has just closed: `[ { ... } ]`
        let mut st_array_close =
            JSONState::Bracket(BracketState::InValue(PrimValue::NestedValueCompleted));
        let res_array_close = parse_char(']', &mut st_array_close);
        assert_eq!(res_array_close, Ok(Token::CloseBracket));

        // Simulates being in an object after a nested array has just closed: `{ "k": [...] ,`
        let mut st_obj_comma =
            JSONState::Brace(BraceState::InValue(PrimValue::NestedValueCompleted));
        let res_obj_comma = parse_char(',', &mut st_obj_comma);
        assert_eq!(res_obj_comma, Ok(Token::Comma));
        assert_eq!(st_obj_comma, JSONState::Brace(BraceState::ExpectingKey));

        // Simulates being in an object after a nested array has just closed: `{ "k": [...] }`
        let mut st_obj_close =
            JSONState::Brace(BraceState::InValue(PrimValue::NestedValueCompleted));
        let res_obj_close = parse_char('}', &mut st_obj_close);
        assert_eq!(res_obj_close, Ok(Token::CloseBrace));
    }
}
