use crate::parse_error_types::JSONParseError;
use crate::state_types::*;

pub fn parse_quote_char(state: &mut JSONState) -> Result<Token, JSONParseError> {
    match state {
        JSONState::Brace(brace_state) => match brace_state {
            BraceState::InKey(string_state) => match string_state {
                StringState::Escaped => {
                    *string_state = StringState::Open;
                    Ok(Token::OpenStringData)
                }
                StringState::Open => {
                    *string_state = StringState::Closed;
                    Ok(Token::CloseKey)
                }
                StringState::Closed => Err(JSONParseError::QuoteCharAfterKeyClose),
            },
            BraceState::InValue(prim_value) => match prim_value {
                PrimValue::String(string_state) => match string_state {
                    StringState::Escaped => {
                        *string_state = StringState::Open;
                        Ok(Token::OpenStringData)
                    }
                    StringState::Open => {
                        *string_state = StringState::Closed;
                        Ok(Token::CloseStringData)
                    }
                    StringState::Closed => Err(JSONParseError::QuoteCharAfterValueClose),
                },
                PrimValue::NonString => Err(JSONParseError::QuoteCharInNonStringData),
            },
            BraceState::ExpectingKey => {
                *brace_state = BraceState::InKey(StringState::Open);
                Ok(Token::OpenKey)
            }
            BraceState::ExpectingValue => {
                *brace_state = BraceState::InValue(PrimValue::String(StringState::Open));
                Ok(Token::OpenStringData)
            }
        },
        JSONState::Bracket(bracket_state) => match bracket_state {
            BracketState::InValue(prim_value) => match prim_value {
                PrimValue::String(string_state) => match string_state {
                    StringState::Escaped => {
                        *string_state = StringState::Open;
                        Ok(Token::OpenStringData)
                    }
                    StringState::Open => {
                        *string_state = StringState::Closed;
                        Ok(Token::CloseStringData)
                    }
                    StringState::Closed => Err(JSONParseError::QuoteCharAfterValueClose),
                },
                PrimValue::NonString => Err(JSONParseError::QuoteCharInNonStringData),
            },
            BracketState::ExpectingValue => {
                *bracket_state = BracketState::InValue(PrimValue::String(StringState::Open));
                Ok(Token::OpenStringData)
            }
        },
        _ => Err(JSONParseError::)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brace_state(state: BraceState) -> JSONState {
        JSONState::Brace(state)
    }

    fn bracket_state(state: BracketState) -> JSONState {
        JSONState::Bracket(state)
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
    }

    #[test]
    fn test_quote_in_brace_in_key_open() {
        let mut state = brace_state(BraceState::InKey(StringState::Open));
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::CloseKey);
    }

    #[test]
    fn test_quote_in_brace_in_key_closed() {
        let mut state = brace_state(BraceState::InKey(StringState::Closed));
        let err = parse_quote_char(&mut state).unwrap_err();
        assert!(matches!(err, JSONParseError::QuoteCharAfterKeyClose));
    }

    #[test]
    fn test_quote_in_brace_in_key_escaped() {
        let mut state = brace_state(BraceState::InKey(StringState::Escaped));
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::OpenStringData);
    }

    #[test]
    fn test_quote_in_brace_in_string_value_open() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Open)));
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::CloseStringData);
    }

    #[test]
    fn test_quote_in_brace_in_string_value_closed() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Closed)));
        let err = parse_quote_char(&mut state).unwrap_err();
        assert!(matches!(err, JSONParseError::QuoteCharAfterValueClose));
    }

    #[test]
    fn test_quote_in_brace_in_string_value_escaped() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Escaped)));
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::OpenStringData);
    }

    #[test]
    fn test_quote_in_brace_in_non_string_value() {
        let mut state = brace_state(BraceState::InValue(PrimValue::NonString));
        let err = parse_quote_char(&mut state).unwrap_err();
        assert!(matches!(err, JSONParseError::QuoteCharInNonStringData));
    }

    #[test]
    fn test_quote_in_bracket_expecting_value() {
        let mut state = bracket_state(BracketState::ExpectingValue);
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::OpenStringData);
    }

    #[test]
    fn test_quote_in_bracket_in_string_value_open() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::String(StringState::Open)));
        let token = parse_quote_char(&mut state).unwrap();
        assert_eq!(token, Token::CloseStringData);
    }

    #[test]
    fn test_quote_in_bracket_in_string_value_closed() {
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
    }

    #[test]
    fn test_quote_in_bracket_in_non_string_value() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::NonString));
        let err = parse_quote_char(&mut state).unwrap_err();
        assert!(matches!(err, JSONParseError::QuoteCharInNonStringData));
    }

    #[test]
    fn test_all_states_covered_no_panic() {
        let states = vec![
            brace_state(BraceState::ExpectingKey),
            brace_state(BraceState::ExpectingValue),
            brace_state(BraceState::InKey(StringState::Open)),
            brace_state(BraceState::InKey(StringState::Closed)),
            brace_state(BraceState::InKey(StringState::Escaped)),
            brace_state(BraceState::InValue(PrimValue::String(StringState::Open))),
            brace_state(BraceState::InValue(PrimValue::String(StringState::Closed))),
            brace_state(BraceState::InValue(PrimValue::String(StringState::Escaped))),
            brace_state(BraceState::InValue(PrimValue::NonString)),
            bracket_state(BracketState::ExpectingValue),
            bracket_state(BracketState::InValue(PrimValue::String(StringState::Open))),
            bracket_state(BracketState::InValue(PrimValue::String(
                StringState::Closed,
            ))),
            bracket_state(BracketState::InValue(PrimValue::String(
                StringState::Escaped,
            ))),
            bracket_state(BracketState::InValue(PrimValue::NonString)),
        ];
        for mut state in states {
            let _ = parse_quote_char(&mut state);
        }
    }
}
