use super::{lexer_types::RecursiveStructureType, JSONParseError, Token};
use crate::parser::state_types::{
    BraceState, BracketState, JSONState, NonStringState, PrimValue, StringState,
};

pub fn parse_bracket(
    brace: RecursiveStructureType,
    current_state: &mut JSONState,
) -> Result<Token, JSONParseError> {
    match brace {
        RecursiveStructureType::Open => {
            // An open bracket is valid only when a value is expected, or at the start.
            match current_state {
                // This is the start of the JSON document.
                JSONState::Pending => {
                    *current_state = JSONState::Bracket(BracketState::Empty);
                    Ok(Token::OpenBracket)
                }
                // This is the start of a nested array, which is a valid value.
                JSONState::Brace(BraceState::ExpectingValue)
                | JSONState::Bracket(BracketState::ExpectingValue) => {
                    *current_state = JSONState::Bracket(BracketState::Empty);
                    Ok(Token::OpenBracket)
                }
                // It's an error to open a brakcet in any other context. This correctly
                // handles states like `ExpectingKey` or `Empty`, because an array
                // cannot be a key in JSON.
                _ => Err(JSONParseError::UnexpectedOpenBracket),
            }
        }
        RecursiveStructureType::Close => {
            // A close bracket is only valid if we are inside a bracket context.
            match current_state {
                JSONState::Bracket(bs) => {
                    // The bracket can be closed if the object is empty, or if the
                    // last thing we saw was a complete value.
                    match bs {
                        // This case allows for empty arrays: `[]`.
                        BracketState::Empty => Ok(Token::CloseBracket),
                        // This case allows for closing after a value.
                        BracketState::InValue(
                            PrimValue::String(StringState::Closed)
                            | PrimValue::NonString(NonStringState::Completable(_)),
                        ) => Ok(Token::CloseBracket),
                        _ => Err(JSONParseError::UnexpectedCloseBracket),
                    }
                }
                // It's a structural error to close a bracket when not in a brakcet context.
                _ => Err(JSONParseError::UnexpectedCloseBracket),
            }
        }
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

    // --- OPEN BRACKET TESTS ---

    #[test]
    fn test_open_bracket_from_pending_state() {
        let mut state = JSONState::Pending;
        let result = parse_bracket(RecursiveStructureType::Open, &mut state);
        assert_eq!(result, Ok(Token::OpenBracket));
        assert_eq!(state, bracket_state(BracketState::Empty));
    }

    #[test]
    fn test_open_bracket_when_expecting_value_in_brace() {
        let mut state = brace_state(BraceState::ExpectingValue);
        let result = parse_bracket(RecursiveStructureType::Open, &mut state);
        assert_eq!(result, Ok(Token::OpenBracket));
        assert_eq!(state, bracket_state(BracketState::Empty));
    }

    #[test]
    fn test_open_bracket_when_expecting_value_in_bracket() {
        let mut state = bracket_state(BracketState::ExpectingValue);
        let result = parse_bracket(RecursiveStructureType::Open, &mut state);
        assert_eq!(result, Ok(Token::OpenBracket));
        assert_eq!(state, bracket_state(BracketState::Empty));
    }

    #[test]
    fn test_error_open_bracket_when_expecting_key() {
        let mut state = brace_state(BraceState::ExpectingKey);
        let result = parse_bracket(RecursiveStructureType::Open, &mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedOpenBracket));
    }

    // --- CLOSE BRACKET TESTS ---

    #[test]
    fn test_close_bracket_in_empty_array() {
        let mut state = bracket_state(BracketState::Empty);
        let result = parse_bracket(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Ok(Token::CloseBracket));
    }

    #[test]
    fn test_error_close_bracket_after_dangling_comma() {
        // This test correctly fails, preventing `[1,2,]`
        let mut state = bracket_state(BracketState::ExpectingValue);
        let result = parse_bracket(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedCloseBracket));
    }

    #[test]
    fn test_close_bracket_after_string_value() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::String(
            StringState::Closed,
        )));
        let result = parse_bracket(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Ok(Token::CloseBracket));
    }

    #[test]
    fn test_close_bracket_after_non_string_value() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::NonString(
            NonStringState::Completable("".to_string()),
        )));
        let result = parse_bracket(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Ok(Token::CloseBracket));
    }

    #[test]
    fn test_error_close_bracket_from_incomplete_non_string_data() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::NonString(
            NonStringState::NonCompletable("".to_string()),
        )));
        let result = parse_bracket(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedCloseBracket));
    }

    #[test]
    fn test_error_close_bracket_from_pending() {
        let mut state = JSONState::Pending;
        let result = parse_bracket(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedCloseBracket));
    }

    #[test]
    fn test_error_close_bracket_in_brace_context() {
        let mut state = brace_state(BraceState::Empty);
        let result = parse_bracket(RecursiveStructureType::Close, &mut state);
        assert_eq!(result, Err(JSONParseError::UnexpectedCloseBracket));
    }
}
