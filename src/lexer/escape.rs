use crate::{
    parser::state_types::{BraceState, BracketState, PrimValue, StringState},
    JSONState,
};

use super::{JSONParseError, Token};

pub fn handle_escape(current_state: &mut JSONState) -> Result<Token, JSONParseError> {
    match current_state {
        JSONState::Brace(bs) => match bs {
            BraceState::InValue(PrimValue::String(StringState::Open)) => {
                *bs = BraceState::InValue(PrimValue::String(StringState::Escaped));
                Ok(Token::OpenStringData)
            }
            BraceState::InKey(StringState::Open) => {
                *bs = BraceState::InKey(StringState::Escaped);
                Ok(Token::OpenStringData)
            }
            _ => Err(JSONParseError::UnexpectedEscape),
        },
        JSONState::Bracket(bs) => match bs {
            BracketState::InValue(PrimValue::String(StringState::Open)) => {
                *bs = BracketState::InValue(PrimValue::String(StringState::Escaped));
                Ok(Token::OpenStringData)
            }
            _ => Err(JSONParseError::UnexpectedEscape),
        },
        _ => Err(JSONParseError::UnexpectedEscape),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::state_types::NonStringState;

    use super::*;

    fn brace_state(state: BraceState) -> JSONState {
        JSONState::Brace(state)
    }

    fn bracket_state(state: BracketState) -> JSONState {
        JSONState::Bracket(state)
    }

    #[test]
    fn test_escape_in_brace_string_value() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Open)));
        let result = handle_escape(&mut state);

        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::String(StringState::Escaped)))
        );
    }

    #[test]
    fn test_escape_in_brace_key() {
        let mut state = brace_state(BraceState::InKey(StringState::Open));
        let result = handle_escape(&mut state);

        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(state, brace_state(BraceState::InKey(StringState::Escaped)));
    }

    #[test]
    fn test_escape_in_bracket_string_value() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::String(StringState::Open)));
        let result = handle_escape(&mut state);

        assert_eq!(result, Ok(Token::OpenStringData));
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::String(
                StringState::Escaped
            )))
        );
    }

    #[test]
    fn test_escape_in_brace_expecting_key() {
        let mut state = brace_state(BraceState::ExpectingKey);
        let result = handle_escape(&mut state);

        assert_eq!(result, Err(JSONParseError::UnexpectedEscape));
    }

    #[test]
    fn test_escape_in_brace_expecting_value() {
        let mut state = brace_state(BraceState::ExpectingValue);
        let result = handle_escape(&mut state);

        assert_eq!(result, Err(JSONParseError::UnexpectedEscape));
    }

    #[test]
    fn test_escape_in_brace_non_string_value() {
        let mut state = brace_state(BraceState::InValue(PrimValue::NonString(
            NonStringState::Completable("".to_string()),
        )));
        let result = handle_escape(&mut state);

        assert_eq!(result, Err(JSONParseError::UnexpectedEscape));
    }

    #[test]
    fn test_escape_in_bracket_expecting_value() {
        let mut state = bracket_state(BracketState::ExpectingValue);
        let result = handle_escape(&mut state);

        assert_eq!(result, Err(JSONParseError::UnexpectedEscape));
    }

    #[test]
    fn test_escape_in_bracket_non_string_value() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::NonString(
            NonStringState::Completable("".to_string()),
        )));
        let result = handle_escape(&mut state);

        assert_eq!(result, Err(JSONParseError::UnexpectedEscape));
    }

    #[test]
    fn test_escape_in_already_escaped_brace_value() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Escaped)));
        let result = handle_escape(&mut state);

        assert_eq!(result, Err(JSONParseError::UnexpectedEscape));
    }

    #[test]
    fn test_escape_in_already_escaped_brace_key() {
        let mut state = brace_state(BraceState::InKey(StringState::Escaped));
        let result = handle_escape(&mut state);

        assert_eq!(result, Err(JSONParseError::UnexpectedEscape));
    }

    #[test]
    fn test_escape_in_already_escaped_bracket_value() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::String(
            StringState::Escaped,
        )));
        let result = handle_escape(&mut state);

        assert_eq!(result, Err(JSONParseError::UnexpectedEscape));
    }

    #[test]
    fn test_escape_state_transitions() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Open)));

        let result1 = handle_escape(&mut state);
        assert_eq!(result1, Ok(Token::OpenStringData));
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::String(StringState::Escaped)))
        );

        let result2 = handle_escape(&mut state);
        assert_eq!(result2, Err(JSONParseError::UnexpectedEscape));
    }
}
