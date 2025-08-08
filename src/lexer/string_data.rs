use crate::lexer::lexer_error_types::JSONParseError;
use crate::lexer::lexer_types::Token;
use crate::parser::state_types::{
    BraceState, BracketState, JSONState, NonStringState, PrimValue, StringState,
};

/// A guard function that checks if the parser is in a state where it is
/// actively consuming characters inside an open string.
pub fn is_string_data(state: &JSONState) -> bool {
    matches!(
        state,
        JSONState::Brace(BraceState::InKey(StringState::Open | StringState::Escaped))
            | JSONState::Brace(BraceState::InValue(PrimValue::String(
                StringState::Open | StringState::Escaped
            )))
            | JSONState::Bracket(BracketState::InValue(PrimValue::String(
                StringState::Open | StringState::Escaped
            )))
    )
}

/// Parses a character that is part of the content of a string literal.
/// This is for the characters between the opening and closing quotes.
pub fn parse_string_data(state: &mut JSONState) -> Result<Token, JSONParseError> {
    match state {
        // Case 1: Character is content inside an open string.
        // The state does not change. We return a non-structural `StringContent` token.
        JSONState::Brace(BraceState::InKey(StringState::Open))
        | JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Open)))
        | JSONState::Bracket(BracketState::InValue(PrimValue::String(StringState::Open))) => {
            Ok(Token::StringContent)
        }

        // Case 2: Character is an escaped literal (e.g., the 'n' in '\n').
        // The state transitions from Escaped back to Open.
        JSONState::Brace(BraceState::InKey(string_state @ StringState::Escaped))
        | JSONState::Brace(BraceState::InValue(PrimValue::String(
            string_state @ StringState::Escaped,
        )))
        | JSONState::Bracket(BracketState::InValue(PrimValue::String(
            string_state @ StringState::Escaped,
        ))) => {
            *string_state = StringState::Open;
            Ok(Token::StringContent)
        }

        // All other states are invalid for generic string data.
        _ => Err(JSONParseError::TokenParseErrorMisc(
            "Unexpected character outside of an open string",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lexer_types::Token;
    use crate::parser::state_types::{
        BraceState, BracketState, JSONState, NonStringState, PrimValue, StringState,
    };

    // Helper functions to create states for tests
    fn brace_state(state: BraceState) -> JSONState {
        JSONState::Brace(state)
    }

    fn bracket_state(state: BracketState) -> JSONState {
        JSONState::Bracket(state)
    }

    // --- VALID CONTENT CASES (STATE DOES NOT CHANGE) ---

    #[test]
    fn test_content_in_open_string_key() {
        let mut state = brace_state(BraceState::InKey(StringState::Open));
        let original_state = state.clone();
        let result = parse_string_data(&mut state);
        assert_eq!(result, Ok(Token::StringContent));
        assert_eq!(state, original_state); // State should not change
    }

    #[test]
    fn test_content_in_open_string_value_in_brace() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Open)));
        let original_state = state.clone();
        let result = parse_string_data(&mut state);
        assert_eq!(result, Ok(Token::StringContent));
        assert_eq!(state, original_state);
    }

    // --- VALID CONTENT CASES (AFTER ESCAPE) ---

    #[test]
    fn test_content_after_escape_in_key() {
        let mut state = brace_state(BraceState::InKey(StringState::Escaped));
        let result = parse_string_data(&mut state);
        assert_eq!(result, Ok(Token::StringContent));
        assert_eq!(state, brace_state(BraceState::InKey(StringState::Open)));
    }

    #[test]
    fn test_content_after_escape_in_value_in_bracket() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::String(
            StringState::Escaped,
        )));
        let result = parse_string_data(&mut state);
        assert_eq!(result, Ok(Token::StringContent));
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::String(StringState::Open)))
        );
    }

    // --- INVALID STATE TRANSITIONS ---

    #[test]
    fn test_error_in_all_non_string_contexts() {
        let invalid_states = vec![
            brace_state(BraceState::ExpectingKey),
            brace_state(BraceState::ExpectingValue),
            bracket_state(BracketState::ExpectingValue),
            brace_state(BraceState::InKey(StringState::Closed)),
            brace_state(BraceState::InValue(PrimValue::String(StringState::Closed))),
            brace_state(BraceState::InValue(PrimValue::NonString(
                NonStringState::Completable(String::from("")),
            ))),
            JSONState::Pending,
        ];

        for mut state in invalid_states {
            let result = parse_string_data(&mut state);
            assert!(result.is_err(), "Should have failed for state: {:?}", state);
        }
    }

    // --- is_string_data GUARD FUNCTION TESTS ---

    #[test]
    fn test_is_string_data_guard_returns_true_for_valid_states() {
        let valid_states = vec![
            brace_state(BraceState::InKey(StringState::Open)),
            brace_state(BraceState::InKey(StringState::Escaped)),
            brace_state(BraceState::InValue(PrimValue::String(StringState::Open))),
            brace_state(BraceState::InValue(PrimValue::String(StringState::Escaped))),
            bracket_state(BracketState::InValue(PrimValue::String(StringState::Open))),
            bracket_state(BracketState::InValue(PrimValue::String(
                StringState::Escaped,
            ))),
        ];
        for state in valid_states {
            assert!(is_string_data(&state), "Should be true for {:?}", state);
        }
    }

    #[test]
    fn test_is_string_data_guard_returns_false_for_invalid_states() {
        let invalid_states = vec![
            brace_state(BraceState::Empty),
            brace_state(BraceState::ExpectingKey),
            brace_state(BraceState::InKey(StringState::Closed)),
            bracket_state(BracketState::ExpectingValue),
            JSONState::Pending,
        ];
        for state in invalid_states {
            assert!(!is_string_data(&state), "Should be false for {:?}", state);
        }
    }
}
