use crate::{
    parser::state_types::{BraceState, BracketState, NonStringState, PrimValue, StringState},
    JSONState,
};

use super::{JSONParseError, Token};

pub fn parse_comma(current_state: &mut JSONState) -> Result<Token, JSONParseError> {
    match current_state {
        // --- Case 1: Comma as a structural separator in an object ---
        // A comma is valid after a completed value, transitioning to expecting the next key.
        JSONState::Brace(BraceState::InValue(
            PrimValue::String(StringState::Closed)
            | PrimValue::NonString(NonStringState::Completable(_)),
        )) => {
            *current_state = JSONState::Brace(BraceState::ExpectingKey);
            Ok(Token::Comma)
        }

        // --- Case 2: Comma as a structural separator in an array ---
        // A comma is valid after a completed value, transitioning to expecting the next value.
        JSONState::Bracket(BracketState::InValue(
            PrimValue::String(StringState::Closed)
            | PrimValue::NonString(NonStringState::Completable(_)),
        )) => {
            *current_state = JSONState::Bracket(BracketState::ExpectingValue);
            Ok(Token::Comma)
        }

        // --- Case 3: Comma as content inside an open string (key or value) ---
        // The comma is just a character within the string; the state does not change.
        JSONState::Brace(BraceState::InKey(StringState::Open))
        | JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Open)))
        | JSONState::Bracket(BracketState::InValue(PrimValue::String(StringState::Open))) => {
            Ok(Token::OpenStringData)
        }

        // --- Case 4: Comma as content after an escape character ---
        // The comma is a literal character, and the string state returns to Open.
        JSONState::Brace(BraceState::InKey(string_state @ StringState::Escaped))
        | JSONState::Brace(BraceState::InValue(PrimValue::String(
            string_state @ StringState::Escaped,
        )))
        | JSONState::Bracket(BracketState::InValue(PrimValue::String(
            string_state @ StringState::Escaped,
        ))) => {
            *string_state = StringState::Open;
            Ok(Token::OpenStringData)
        }

        // --- Case 5: All other states are invalid for a comma ---
        _ => Err(JSONParseError::UnexpectedComma),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper functions to create states for tests
    fn brace_state(state: BraceState) -> JSONState {
        JSONState::Brace(state)
    }

    fn bracket_state(state: BracketState) -> JSONState {
        JSONState::Bracket(state)
    }

    // --- VALID SEPARATOR CASES ---

    #[test]
    fn test_separator_in_brace_after_closed_string_value() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Closed)));
        let result = parse_comma(&mut state);
        assert_eq!(result, Ok(Token::Comma));
        assert_eq!(state, brace_state(BraceState::ExpectingKey));
    }

    #[test]
    fn test_separator_in_brace_after_non_string_value() {
        let mut state = brace_state(BraceState::InValue(PrimValue::NonString(
            NonStringState::Completable("".to_string()),
        )));
        let result = parse_comma(&mut state);
        assert_eq!(result, Ok(Token::Comma));
        assert_eq!(state, brace_state(BraceState::ExpectingKey));
    }

    #[test]
    fn test_separator_in_bracket_after_closed_string_value() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::String(
            StringState::Closed,
        )));
        let result = parse_comma(&mut state);
        assert_eq!(result, Ok(Token::Comma));
        assert_eq!(state, bracket_state(BracketState::ExpectingValue));
    }

    #[test]
    fn test_separator_in_bracket_after_non_string_value() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::NonString(
            NonStringState::Completable("".to_string()),
        )));
        let result = parse_comma(&mut state);
        assert_eq!(result, Ok(Token::Comma));
        assert_eq!(state, bracket_state(BracketState::ExpectingValue));
    }

    // --- VALID CONTENT CASES (COMMA INSIDE A STRING) ---

    #[test]
    fn test_content_in_open_string_key() {
        let mut state = brace_state(BraceState::InKey(StringState::Open));
        let original_state = state.clone();
        let result = parse_comma(&mut state);
        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(state, original_state); // State should not change
    }

    #[test]
    fn test_content_in_open_string_value_in_brace() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Open)));
        let original_state = state.clone();
        let result = parse_comma(&mut state);
        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(state, original_state);
    }

    #[test]
    fn test_content_in_open_string_value_in_bracket() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::String(StringState::Open)));
        let original_state = state.clone();
        let result = parse_comma(&mut state);
        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(state, original_state);
    }

    // --- VALID CONTENT CASES (AFTER ESCAPE) ---

    #[test]
    fn test_content_after_escape_in_key() {
        let mut state = brace_state(BraceState::InKey(StringState::Escaped));
        let result = parse_comma(&mut state);
        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(state, brace_state(BraceState::InKey(StringState::Open)));
    }

    #[test]
    fn test_content_after_escape_in_value_in_brace() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Escaped)));
        let result = parse_comma(&mut state);
        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::String(StringState::Open)))
        );
    }

    #[test]
    fn test_content_after_escape_in_value_in_bracket() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::String(
            StringState::Escaped,
        )));
        let result = parse_comma(&mut state);
        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::String(StringState::Open)))
        );
    }

    // --- INVALID STATE TRANSITIONS ---

    #[test]
    fn test_error_comma_in_brace_expecting_key() {
        let mut state = brace_state(BraceState::ExpectingKey);
        let result = parse_comma(&mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedComma));
    }

    #[test]
    fn test_error_comma_in_brace_expecting_value() {
        let mut state = brace_state(BraceState::ExpectingValue);
        let result = parse_comma(&mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedComma));
    }

    #[test]
    fn test_error_comma_in_bracket_expecting_value() {
        let mut state = bracket_state(BracketState::ExpectingValue);
        let result = parse_comma(&mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedComma));
    }

    #[test]
    fn test_error_comma_after_closed_key() {
        let mut state = brace_state(BraceState::InKey(StringState::Closed));
        let result = parse_comma(&mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedComma));
    }
}
