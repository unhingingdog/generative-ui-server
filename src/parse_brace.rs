use crate::parse_error_types::JSONParseError;
use crate::state_types::*;
use crate::structure_type::RecursiveStructureType;

pub fn parse_brace(
    brace: RecursiveStructureType,
    current_state: &mut JSONState,
) -> Result<Token, JSONParseError> {
    match brace {
        RecursiveStructureType::Open => {
            match current_state {
                // This is the start of the JSON document.
                JSONState::Pending => {
                    *current_state = JSONState::Brace(BraceState::Empty);
                    Ok(Token::OpenBrace)
                }
                // This is the start of a nested object.
                JSONState::Brace(BraceState::ExpectingValue)
                | JSONState::Bracket(BracketState::ExpectingValue) => {
                    *current_state = JSONState::Brace(BraceState::Empty);
                    Ok(Token::OpenBrace)
                }
                // It's an error to open a brace in any other context.
                _ => Err(JSONParseError::UnexpectedOpenBrace),
            }
        }
        RecursiveStructureType::Close => {
            // A close brace is only valid if we are inside a brace context.
            match current_state {
                JSONState::Brace(bs) => {
                    // The brace can be closed if the object is empty, or if the
                    // last thing we saw was a complete value.
                    match bs {
                        // This case allows for empty objects: `{}`.
                        BraceState::Empty => Ok(Token::CloseBrace),
                        // This case allows for closing after a key-value pair.
                        BraceState::InValue(PrimValue::String(StringState::Closed))
                        | BraceState::InValue(PrimValue::NonString) => Ok(Token::CloseBrace),
                        // this will reject dangling commas
                        _ => Err(JSONParseError::UnexpectedCloseBrace),
                    }
                }
                // It's a structural error to close a brace when not in a brace context.
                _ => Err(JSONParseError::UnexpectedCloseBrace),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_error_types::JSONParseError; // Make sure this is imported for tests

    // Helper functions to create states for tests
    fn brace_state(state: BraceState) -> JSONState {
        JSONState::Brace(state)
    }

    fn bracket_state(state: BracketState) -> JSONState {
        JSONState::Bracket(state)
    }

    // --- OPEN BRACE TESTS ---

    #[test]
    fn test_open_brace_from_pending_state() {
        let mut state = JSONState::Pending;
        let result = parse_brace(RecursiveStructureType::Open, &mut state);
        assert_eq!(result, Ok(Token::OpenBrace));
        assert_eq!(state, brace_state(BraceState::Empty));
    }

    #[test]
    fn test_open_brace_when_expecting_value_in_brace() {
        let mut state = brace_state(BraceState::ExpectingValue);
        let result = parse_brace(RecursiveStructureType::Open, &mut state);
        assert_eq!(result, Ok(Token::OpenBrace));
        assert_eq!(state, brace_state(BraceState::Empty));
    }

    #[test]
    fn test_open_brace_when_expecting_value_in_bracket() {
        let mut state = bracket_state(BracketState::ExpectingValue);
        let result = parse_brace(RecursiveStructureType::Open, &mut state);
        assert_eq!(result, Ok(Token::OpenBrace));
        assert_eq!(state, brace_state(BraceState::Empty));
    }

    #[test]
    fn test_error_open_brace_in_string_key() {
        let mut state = brace_state(BraceState::InKey(StringState::Open));
        let result = parse_brace(RecursiveStructureType::Open, &mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedOpenBrace));
    }

    // --- CLOSE BRACE TESTS ---

    #[test]
    fn test_close_brace_in_empty_object() {
        let mut state = brace_state(BraceState::Empty);
        let result = parse_brace(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Ok(Token::CloseBrace));
    }

    #[test]
    fn test_error_close_brace_after_dangling_comma() {
        // This test now correctly fails, preventing `{"key":"val",}`
        let mut state = brace_state(BraceState::ExpectingKey);
        let result = parse_brace(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedCloseBrace));
    }

    #[test]
    fn test_close_brace_after_string_value() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Closed)));
        let result = parse_brace(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Ok(Token::CloseBrace));
    }

    #[test]
    fn test_close_brace_after_non_string_value() {
        let mut state = brace_state(BraceState::InValue(PrimValue::NonString));
        let result = parse_brace(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Ok(Token::CloseBrace));
    }

    #[test]
    fn test_error_close_brace_when_expecting_value() {
        let mut state = brace_state(BraceState::ExpectingValue);
        let result = parse_brace(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedCloseBrace));
    }

    #[test]
    fn test_error_close_brace_from_pending() {
        let mut state = JSONState::Pending;
        let result = parse_brace(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedCloseBrace));
    }

    #[test]
    fn test_error_close_brace_in_bracket_context() {
        let mut state = bracket_state(BracketState::ExpectingValue);
        let result = parse_brace(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedCloseBrace));
    }
}
