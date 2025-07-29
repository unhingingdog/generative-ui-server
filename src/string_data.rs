use crate::parse_error_types::JSONParseError;
use crate::state_types::*;

pub fn is_string_data(state: &JSONState) -> bool {
    matches!(
        state,
        JSONState::Brace(BraceState::InKey(StringState::Open))
            | JSONState::Brace(BraceState::InKey(StringState::Escaped))
            | JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Open)))
            | JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Escaped)))
            | JSONState::Bracket(BracketState::InValue(PrimValue::String(StringState::Open)))
            | JSONState::Bracket(BracketState::InValue(PrimValue::String(
                StringState::Escaped
            )))
    )
}

pub fn parse_string_data(state: &mut JSONState) -> Result<Token, JSONParseError> {
    match state {
        JSONState::Brace(bs) => match bs {
            // If we're in an open string (as a key or a value), the character is simply content.
            // The state does not change.
            BraceState::InKey(StringState::Open)
            | BraceState::InValue(PrimValue::String(StringState::Open)) => {
                Ok(Token::OpenStringData)
            }
            // If the previous character was an escape (`\`), this character is the escaped literal.
            // We transition the state back to `Open` as the escape sequence is now complete.
            BraceState::InKey(StringState::Escaped) => {
                *bs = BraceState::InKey(StringState::Open);
                Ok(Token::OpenStringData)
            }
            BraceState::InValue(PrimValue::String(StringState::Escaped)) => {
                *bs = BraceState::InValue(PrimValue::String(StringState::Open));
                Ok(Token::OpenStringData)
            }
            // If we are not inside an open string, receiving a generic character is a syntax error.
            _ => Err(JSONParseError::TokenParseErrorMisc(
                "Unexpected character in object context",
            )),
        },
        JSONState::Bracket(bs) => match bs {
            // If we're in an open string value, the character is content. The state doesn't change.
            BracketState::InValue(PrimValue::String(StringState::Open)) => {
                Ok(Token::OpenStringData)
            }
            // If the previous character was an escape, this character is the literal.
            // Transition back to the `Open` state.
            BracketState::InValue(PrimValue::String(StringState::Escaped)) => {
                *bs = BracketState::InValue(PrimValue::String(StringState::Open));
                Ok(Token::OpenStringData)
            }
            // If we are not inside an open string, receiving a generic character is a syntax error.
            _ => Err(JSONParseError::TokenParseErrorMisc(
                "Unexpected character in array context",
            )),
        },
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

    // --- VALID CONTENT CASES (STATE DOES NOT CHANGE) ---

    #[test]
    fn test_content_in_open_string_key() {
        let mut state = brace_state(BraceState::InKey(StringState::Open));
        let original_state = state.clone();
        let result = parse_string_data(&mut state);
        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(state, original_state); // State should not change
    }

    #[test]
    fn test_content_in_open_string_value_in_brace() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Open)));
        let original_state = state.clone();
        let result = parse_string_data(&mut state);
        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(state, original_state);
    }

    #[test]
    fn test_content_in_open_string_value_in_bracket() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::String(StringState::Open)));
        let original_state = state.clone();
        let result = parse_string_data(&mut state);
        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(state, original_state);
    }

    // --- VALID CONTENT CASES (AFTER ESCAPE) ---

    #[test]
    fn test_content_after_escape_in_key() {
        let mut state = brace_state(BraceState::InKey(StringState::Escaped));
        let result = parse_string_data(&mut state);
        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(state, brace_state(BraceState::InKey(StringState::Open)));
    }

    #[test]
    fn test_content_after_escape_in_value_in_brace() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Escaped)));
        let result = parse_string_data(&mut state);
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
        let result = parse_string_data(&mut state);
        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::String(StringState::Open)))
        );
    }

    // --- INVALID STATE TRANSITIONS ---

    #[test]
    fn test_error_in_brace_expecting_key() {
        let mut state = brace_state(BraceState::ExpectingKey);
        let result = parse_string_data(&mut state);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_in_brace_expecting_value() {
        let mut state = brace_state(BraceState::ExpectingValue);
        let result = parse_string_data(&mut state);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_in_bracket_expecting_value() {
        let mut state = bracket_state(BracketState::ExpectingValue);
        let result = parse_string_data(&mut state);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_after_closed_key() {
        let mut state = brace_state(BraceState::InKey(StringState::Closed));
        let result = parse_string_data(&mut state);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_after_closed_string_value() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Closed)));
        let result = parse_string_data(&mut state);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_in_non_string_value() {
        let mut state = brace_state(BraceState::InValue(PrimValue::NonString));
        let result = parse_string_data(&mut state);
        assert!(result.is_err());
    }
}
