use crate::{
    parser::state_types::{BraceState, BracketState, PrimValue, StringState},
    JSONState,
};

use super::{JSONParseError, Token};

pub fn parse_colon(current_state: &mut JSONState) -> Result<Token, JSONParseError> {
    match current_state {
        JSONState::Brace(bs) => match bs {
            // Case 1: The only valid place for a structural colon is after a closed key.
            BraceState::InKey(StringState::Closed) => {
                *bs = BraceState::ExpectingValue;
                Ok(Token::Colon)
            }

            // Case 2: Colon is content inside an open string (key or value).
            BraceState::InKey(StringState::Open)
            | BraceState::InValue(PrimValue::String(StringState::Open)) => {
                // State does not change, it's just a character in the string.
                Ok(Token::OpenStringData)
            }

            // Case 3: Colon is content after an escape char.
            BraceState::InKey(StringState::Escaped)
            | BraceState::InValue(PrimValue::String(StringState::Escaped)) => {
                *bs = BraceState::InKey(StringState::Open);
                Ok(Token::OpenStringData)
            }

            // Case 4: All other states within a brace are invalid for a colon.
            _ => Err(JSONParseError::UnexpectedColon),
        },
        // A colon is never valid inside a Bracket context, unless it's within a string.
        JSONState::Bracket(bs) => match bs {
            BracketState::InValue(PrimValue::String(StringState::Open)) => {
                // State does not change, it's just a character in the string.
                Ok(Token::OpenStringData)
            }
            BracketState::InValue(PrimValue::String(StringState::Escaped)) => {
                *bs = BracketState::InValue(PrimValue::String(StringState::Open));
                Ok(Token::OpenStringData)
            }
            _ => Err(JSONParseError::UnexpectedColon),
        },
        _ => Err(JSONParseError::UnexpectedColon),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::state_types::NonStringState;

    use super::*;

    // Helper functions to create states for tests
    fn brace_state(state: BraceState) -> JSONState {
        JSONState::Brace(state)
    }

    fn bracket_state(state: BracketState) -> JSONState {
        JSONState::Bracket(state)
    }

    // --- VALID STATE TRANSITION ---

    #[test]
    fn test_colon_after_closed_key() {
        let mut state = brace_state(BraceState::InKey(StringState::Closed));
        let result = parse_colon(&mut state);
        assert_eq!(result, Ok(Token::Colon));
        assert_eq!(state, brace_state(BraceState::ExpectingValue));
    }

    // --- VALID CONTENT CASES (COLON INSIDE A STRING) ---

    #[test]
    fn test_colon_in_open_string_key() {
        let mut state = brace_state(BraceState::InKey(StringState::Open));
        let original_state = state.clone();
        let result = parse_colon(&mut state);
        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(state, original_state); // State should not change
    }

    #[test]
    fn test_colon_in_open_string_value_in_brace() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Open)));
        let original_state = state.clone();
        let result = parse_colon(&mut state);
        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(state, original_state);
    }

    #[test]
    fn test_colon_in_open_string_value_in_bracket() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::String(StringState::Open)));
        let original_state = state.clone();
        let result = parse_colon(&mut state);
        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(state, original_state);
    }

    // --- INVALID STATE TRANSITIONS ---

    #[test]
    fn test_error_colon_in_brace_expecting_key() {
        let mut state = brace_state(BraceState::ExpectingKey);
        let result = parse_colon(&mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedColon));
    }

    #[test]
    fn test_error_colon_in_brace_expecting_value() {
        let mut state = brace_state(BraceState::ExpectingValue);
        let result = parse_colon(&mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedColon));
    }

    #[test]
    fn test_error_colon_after_string_value() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Closed)));
        let result = parse_colon(&mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedColon));
    }

    #[test]
    fn test_error_colon_after_non_string_value() {
        let mut state = brace_state(BraceState::InValue(PrimValue::NonString(
            NonStringState::Completable("".to_string()),
        )));
        let result = parse_colon(&mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedColon));
    }

    #[test]
    fn test_error_colon_in_any_bracket_state() {
        let states = vec![
            bracket_state(BracketState::ExpectingValue),
            bracket_state(BracketState::InValue(PrimValue::NonString(
                NonStringState::Completable("".to_string()),
            ))),
            bracket_state(BracketState::InValue(PrimValue::String(
                StringState::Closed,
            ))),
        ];
        for mut state in states {
            let result = parse_colon(&mut state);
            assert_eq!(result, Err(JSONParseError::UnexpectedColon));
        }
    }
}
