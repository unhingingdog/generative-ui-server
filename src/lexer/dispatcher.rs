use crate::JSONState;

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

pub fn parse_char(c: char, current_state: &mut JSONState) -> Result<Token, JSONParseError> {
    match c {
        // Chars that have specific meaning when inside a string.
        '\\' => handle_escape(current_state),
        '"' => parse_quote_char(current_state),

        // Data-only handling
        _ if is_string_data(current_state) => parse_string_data(current_state),
        c if is_non_string_data(c, current_state) => parse_non_string_data(c, current_state),

        // Structural token handling
        '{' => parse_brace(RecursiveStructureType::Open, current_state),
        '}' => parse_brace(RecursiveStructureType::Close, current_state),
        '[' => parse_bracket(RecursiveStructureType::Open, current_state),
        ']' => parse_bracket(RecursiveStructureType::Close, current_state),
        ':' => parse_colon(current_state),
        ',' => parse_comma(current_state),

        // ignored whitespace
        ' ' | '\t' | '\n' | '\r' => Ok(Token::Whitespace),

        // bad
        _ => Err(JSONParseError::InvalidCharEncountered),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::state_types::{BraceState, PrimValue, StringState};

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

        assert_eq!(result, Ok(Token::OpenStringData)); // `handle_escape` returns this
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
}
