use crate::parse_error_types::JSONParseError;
use crate::state_types::*;

pub fn parse_comma(current_state: &mut JSONState) -> Result<Token, JSONParseError> {
    match current_state {
        JSONState::Brace(bs) => match bs {
            // Case 1: Comma is content inside an open string (key). State does not change.
            BraceState::InKey(StringState::Open) => Ok(Token::OpenStringData),

            // Case 2: Comma is content after an escape char in a key. State returns to Open.
            BraceState::InKey(StringState::Escaped) => {
                *bs = BraceState::InKey(StringState::Open);
                Ok(Token::OpenStringData)
            }

            // Case 3: Comma is content inside an open string (value). State does not change.
            BraceState::InValue(PrimValue::String(StringState::Open)) => Ok(Token::OpenStringData),

            // Case 4: Comma is content after an escape char in a value. State returns to Open.
            BraceState::InValue(PrimValue::String(StringState::Escaped)) => {
                *bs = BraceState::InValue(PrimValue::String(StringState::Open));
                Ok(Token::OpenStringData)
            }

            // Case 5: Comma is a separator after a completed value. State transitions to ExpectingKey.
            BraceState::InValue(PrimValue::String(StringState::Closed))
            | BraceState::InValue(PrimValue::NonString) => {
                *bs = BraceState::ExpectingKey;
                Ok(Token::Comma)
            }

            // Case 6: All other states are invalid for a comma.
            _ => Err(JSONParseError::UnexpectedComma),
        },
        JSONState::Bracket(bs) => match bs {
            // Case 1: Comma is content inside an open string. State does not change.
            BracketState::InValue(PrimValue::String(StringState::Open)) => {
                Ok(Token::OpenStringData)
            }

            // Case 2: Comma is content after an escape char. State returns to Open.
            BracketState::InValue(PrimValue::String(StringState::Escaped)) => {
                *bs = BracketState::InValue(PrimValue::String(StringState::Open));
                Ok(Token::OpenStringData)
            }

            // Case 3: Comma is a separator after a completed value. State transitions to ExpectingValue.
            BracketState::InValue(PrimValue::String(StringState::Closed))
            | BracketState::InValue(PrimValue::NonString) => {
                *bs = BracketState::ExpectingValue;
                Ok(Token::Comma)
            }

            // Case 4: All other states are invalid for a comma.
            _ => Err(JSONParseError::UnexpectedComma),
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
        let mut state = brace_state(BraceState::InValue(PrimValue::NonString));
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
        let mut state = bracket_state(BracketState::InValue(PrimValue::NonString));
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
