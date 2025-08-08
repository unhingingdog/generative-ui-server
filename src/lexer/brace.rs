use super::{lexer_types::RecursiveStructureType, JSONParseError, Token};
use crate::parser::state_types::{
    BraceState, BracketState, JSONState, NonStringState, PrimValue, StringState,
};

pub fn parse_brace(
    brace: RecursiveStructureType,
    current_state: &mut JSONState,
) -> Result<Token, JSONParseError> {
    match brace {
        RecursiveStructureType::Open => {
            // An open brace is valid only when a value is expected, or at the start.
            match current_state {
                // This is the start of the JSON document.
                JSONState::Pending => {
                    *current_state = JSONState::Brace(BraceState::Empty);
                    Ok(Token::OpenBrace)
                }
                // This is the start of a nested object, which is a valid value.
                JSONState::Brace(BraceState::ExpectingValue)
                | JSONState::Bracket(BracketState::Empty | BracketState::ExpectingValue) => {
                    *current_state = JSONState::Brace(BraceState::Empty);
                    Ok(Token::OpenBrace)
                }
                // It's an error to open a brace in any other context. This correctly
                // handles states like `ExpectingKey` or `Empty`, because an object
                // cannot be a key in JSON.
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
                        BraceState::InValue(
                            PrimValue::String(StringState::Closed)
                            | PrimValue::NonString(NonStringState::Completable(_)),
                        ) => Ok(Token::CloseBrace),
                        // This will reject dangling commas (from ExpectingKey) and all other invalid states.
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
    use crate::lexer::lexer_error_types::JSONParseError;
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
    fn test_open_brace_in_empty_bracket() {
        // This test specifically covers the bug fix.
        let mut state = bracket_state(BracketState::Empty);
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
        let mut state = brace_state(BraceState::InValue(PrimValue::NonString(
            NonStringState::Completable("".to_string()),
        )));
        let result = parse_brace(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Ok(Token::CloseBrace));
    }

    #[test]
    fn test_error_close_brace_when_in_non_completable_non_string_data() {
        let mut state = brace_state(BraceState::InValue(PrimValue::NonString(
            NonStringState::NonCompletable("".to_string()),
        )));
        let result = parse_brace(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedCloseBrace));
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
