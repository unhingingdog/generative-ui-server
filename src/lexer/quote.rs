use crate::{
    parser::state_types::{BraceState, BracketState, NonStringState, PrimValue, StringState},
    JSONState,
};

use super::{JSONParseError, Token};

pub fn parse_quote_char(state: &mut JSONState) -> Result<Token, JSONParseError> {
    match state {
        // --- Case 1: Start of a new Key ---
        // A quote can start a key if the object is empty or expecting a key after a comma.
        JSONState::Brace(BraceState::Empty | BraceState::ExpectingKey) => {
            *state = JSONState::Brace(BraceState::InKey(StringState::Open));
            Ok(Token::OpenKey)
        }

        // --- Case 2: Start of a new String Value ---
        // A quote can start a value if one is expected in an object or an array.
        JSONState::Brace(BraceState::ExpectingValue) => {
            *state = JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Open)));
            Ok(Token::OpenStringData)
        }
        JSONState::Bracket(BracketState::Empty | BracketState::ExpectingValue) => {
            *state =
                JSONState::Bracket(BracketState::InValue(PrimValue::String(StringState::Open)));
            Ok(Token::OpenStringData)
        }

        // --- Case 3: Inside an open Key string ---
        // This handles closing the key or handling an escaped quote.
        JSONState::Brace(BraceState::InKey(string_state)) => match string_state {
            StringState::Open => {
                *string_state = StringState::Closed;
                Ok(Token::CloseKey)
            }
            StringState::Escaped => {
                *string_state = StringState::Open;
                Ok(Token::OpenStringData)
            }
            StringState::Closed => Err(JSONParseError::QuoteCharAfterKeyClose),
        },

        // --- Case 4: Inside an open Value string (in either a Brace or Bracket) ---
        // This handles closing the value or handling an escaped quote.
        JSONState::Brace(BraceState::InValue(PrimValue::String(string_state)))
        | JSONState::Bracket(BracketState::InValue(PrimValue::String(string_state))) => {
            match string_state {
                StringState::Open => {
                    *string_state = StringState::Closed;
                    Ok(Token::CloseStringData)
                }
                StringState::Escaped => {
                    *string_state = StringState::Open;
                    Ok(Token::OpenStringData)
                }
                StringState::Closed => Err(JSONParseError::QuoteCharAfterValueClose),
            }
        }

        // --- Case 5: Error conditions ---
        // A quote is invalid if we're in the middle of a number/literal, popping out of a nested
        // value, or at the very start.
        JSONState::Brace(BraceState::InValue(
            PrimValue::NonString(_) | PrimValue::NestedValueCompleted,
        ))
        | JSONState::Bracket(BracketState::InValue(
            PrimValue::NonString(_) | PrimValue::NestedValueCompleted,
        )) => Err(JSONParseError::QuoteCharInNonStringData),

        JSONState::Pending => Err(JSONParseError::UnexpectedQuoteChar),
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
    fn test_error_quote_after_nested_value_completed() {
        let mut state_in_brace = brace_state(BraceState::InValue(PrimValue::NestedValueCompleted));
        let err_brace = parse_quote_char(&mut state_in_brace).unwrap_err();
        assert!(matches!(
            err_brace,
            JSONParseError::QuoteCharInNonStringData
        ));

        let mut state_in_bracket =
            bracket_state(BracketState::InValue(PrimValue::NestedValueCompleted));
        let err_bracket = parse_quote_char(&mut state_in_bracket).unwrap_err();
        assert!(matches!(
            err_bracket,
            JSONParseError::QuoteCharInNonStringData
        ));
    }

    #[test]
    fn test_quote_in_brace_empty() {
        let mut state = brace_state(BraceState::Empty);
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::OpenKey);
        assert_eq!(state, brace_state(BraceState::InKey(StringState::Open)));
    }

    #[test]
    fn test_quote_in_brace_expecting_key() {
        let mut state = brace_state(BraceState::ExpectingKey);
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::OpenKey);
        assert_eq!(state, brace_state(BraceState::InKey(StringState::Open)));
    }

    #[test]
    fn test_quote_in_brace_expecting_value() {
        let mut state = brace_state(BraceState::ExpectingValue);
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::OpenStringData);
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::String(StringState::Open)))
        );
    }

    #[test]
    fn test_quote_in_brace_in_key_open() {
        let mut state = brace_state(BraceState::InKey(StringState::Open));
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::CloseKey);
        assert_eq!(state, brace_state(BraceState::InKey(StringState::Closed)));
    }

    #[test]
    fn test_error_quote_in_brace_in_key_closed() {
        let mut state = brace_state(BraceState::InKey(StringState::Closed));
        let err = parse_quote_char(&mut state).unwrap_err();
        assert!(matches!(err, JSONParseError::QuoteCharAfterKeyClose));
    }

    #[test]
    fn test_quote_in_brace_in_key_escaped() {
        let mut state = brace_state(BraceState::InKey(StringState::Escaped));
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::OpenStringData);
        assert_eq!(state, brace_state(BraceState::InKey(StringState::Open)));
    }

    #[test]
    fn test_quote_in_brace_in_string_value_open() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Open)));
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::CloseStringData);
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::String(StringState::Closed)))
        );
    }

    #[test]
    fn test_error_quote_in_brace_in_string_value_closed() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Closed)));
        let err = parse_quote_char(&mut state).unwrap_err();
        assert!(matches!(err, JSONParseError::QuoteCharAfterValueClose));
    }

    #[test]
    fn test_quote_in_brace_in_string_value_escaped() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Escaped)));
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::OpenStringData);
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::String(StringState::Open)))
        );
    }

    #[test]
    fn test_error_quote_in_brace_in_non_string_value() {
        let mut state = brace_state(BraceState::InValue(PrimValue::NonString(
            NonStringState::Completable(String::from("")),
        )));
        let err = parse_quote_char(&mut state).unwrap_err();
        assert!(matches!(err, JSONParseError::QuoteCharInNonStringData));
    }

    #[test]
    fn test_quote_in_bracket_empty() {
        let mut state = bracket_state(BracketState::Empty);
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::OpenStringData);
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::String(StringState::Open)))
        );
    }

    #[test]
    fn test_quote_in_bracket_expecting_value() {
        let mut state = bracket_state(BracketState::ExpectingValue);
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::OpenStringData);
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::String(StringState::Open)))
        );
    }

    #[test]
    fn test_quote_in_bracket_in_string_value_open() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::String(StringState::Open)));
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::CloseStringData);
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::String(
                StringState::Closed
            )))
        );
    }

    #[test]
    fn test_error_quote_in_bracket_in_string_value_closed() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::String(
            StringState::Closed,
        )));
        let err = parse_quote_char(&mut state).unwrap_err();
        assert!(matches!(err, JSONParseError::QuoteCharAfterValueClose));
    }

    #[test]
    fn test_quote_in_bracket_in_string_value_escaped() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::String(
            StringState::Escaped,
        )));
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::OpenStringData);
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::String(StringState::Open)))
        );
    }

    #[test]
    fn test_error_quote_in_bracket_in_non_string_value() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::NonString(
            NonStringState::Completable(String::from("")),
        )));
        let err = parse_quote_char(&mut state).unwrap_err();
        assert!(matches!(err, JSONParseError::QuoteCharInNonStringData));
    }

    #[test]
    fn test_error_quote_from_pending() {
        let mut state = JSONState::Pending;
        let err = parse_quote_char(&mut state).unwrap_err();
        assert!(matches!(err, JSONParseError::UnexpectedQuoteChar));
    }

    #[test]
    fn test_all_states_covered_no_panic() {
        let states = vec![
            brace_state(BraceState::Empty),
            brace_state(BraceState::ExpectingKey),
            brace_state(BraceState::ExpectingValue),
            brace_state(BraceState::InKey(StringState::Open)),
            brace_state(BraceState::InKey(StringState::Closed)),
            brace_state(BraceState::InKey(StringState::Escaped)),
            brace_state(BraceState::InValue(PrimValue::String(StringState::Open))),
            brace_state(BraceState::InValue(PrimValue::String(StringState::Closed))),
            brace_state(BraceState::InValue(PrimValue::String(StringState::Escaped))),
            brace_state(BraceState::InValue(PrimValue::NonString(
                NonStringState::Completable(String::from("")),
            ))),
            brace_state(BraceState::InValue(PrimValue::NonString(
                NonStringState::NonCompletable(String::from("")),
            ))),
            bracket_state(BracketState::Empty),
            bracket_state(BracketState::ExpectingValue),
            bracket_state(BracketState::InValue(PrimValue::String(StringState::Open))),
            bracket_state(BracketState::InValue(PrimValue::String(
                StringState::Closed,
            ))),
            bracket_state(BracketState::InValue(PrimValue::String(
                StringState::Escaped,
            ))),
            brace_state(BraceState::InValue(PrimValue::NonString(
                NonStringState::Completable(String::from("")),
            ))),
            JSONState::Pending,
        ];
        for mut state in states {
            // The result is not important, we just want to ensure no panics.
            let _ = parse_quote_char(&mut state);
        }
    }
}
