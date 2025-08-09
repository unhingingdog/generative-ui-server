use crate::{
    parser::state_types::{BraceState, BracketState, NonStringState, PrimValue},
    JSONState,
};

use super::{
    is_valid_non_string_data::{is_non_valid_non_string_data, CompletionCheckValues},
    JSONParseError, Token,
};

fn is_non_string_start(c: char) -> bool {
    c.is_ascii_digit() || c == '-' || matches!(c, 'n' | 't' | 'f')
}

pub fn is_non_string_data(c: char, state: &JSONState) -> bool {
    match state {
        // States where a new non-string value can start.
        JSONState::Brace(BraceState::ExpectingValue)
        | JSONState::Bracket(BracketState::Empty | BracketState::ExpectingValue) => {
            is_non_string_start(c)
        }
        // States where we are already inside a non-string value.
        JSONState::Brace(BraceState::InValue(PrimValue::NonString(_)))
        | JSONState::Bracket(BracketState::InValue(PrimValue::NonString(_))) => true,
        _ => false,
    }
}

pub fn parse_non_string_data(c: char, state: &mut JSONState) -> Result<Token, JSONParseError> {
    match state {
        // --- Case 1: Starting a new non-string value ---
        JSONState::Brace(bs @ BraceState::ExpectingValue) => {
            let s = c.to_string();
            *bs = BraceState::InValue(PrimValue::NonString(if c == '-' {
                NonStringState::NonCompletable(s)
            } else {
                NonStringState::Completable(s)
            }));
            Ok(Token::NonStringData)
        }
        JSONState::Bracket(bs @ (BracketState::Empty | BracketState::ExpectingValue)) => {
            let s = c.to_string();
            *bs = BracketState::InValue(PrimValue::NonString(if c == '-' {
                NonStringState::NonCompletable(s)
            } else {
                NonStringState::Completable(s)
            }));
            Ok(Token::NonStringData)
        }

        // --- Case 2: Continuing an existing non-string value ---
        JSONState::Brace(BraceState::InValue(PrimValue::NonString(ns_state)))
        | JSONState::Bracket(BracketState::InValue(PrimValue::NonString(ns_state))) => {
            // Borrow the current buffer
            let buffer = match ns_state {
                NonStringState::Completable(s) | NonStringState::NonCompletable(s) => s,
            };

            let status = is_non_valid_non_string_data(c, buffer);
            buffer.push(c);
            *ns_state = match status {
                Ok(CompletionCheckValues::Complete) => NonStringState::Completable(buffer.clone()),
                // Incomplete or Err => not closable yet
                _ => NonStringState::NonCompletable(buffer.clone()),
            };
            status.map(|_| Token::NonStringData)
        }

        _ => Err(JSONParseError::UnexpectedCharInNonStringData),
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

    // --- Start Parsing Tests ---

    #[test]
    fn test_start_literal_in_bracket() {
        let mut state = bracket_state(BracketState::Empty);
        let result = parse_non_string_data('t', &mut state);
        assert_eq!(result, Ok(Token::NonStringData));
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::NonString(
                NonStringState::Completable("t".to_string())
            )))
        );
    }

    #[test]
    fn test_start_number_in_brace() {
        let mut state = brace_state(BraceState::ExpectingValue);
        let result = parse_non_string_data('1', &mut state);
        assert_eq!(result, Ok(Token::NonStringData));
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::NonString(
                NonStringState::Completable("1".to_string())
            )))
        );
    }

    #[test]
    fn test_start_minus_in_brace_sets_noncompletable() {
        let mut state = brace_state(BraceState::ExpectingValue);
        let result = parse_non_string_data('-', &mut state);
        assert_eq!(result, Ok(Token::NonStringData));
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::NonString(
                NonStringState::NonCompletable("-".to_string())
            )))
        );
    }

    #[test]
    fn test_start_minus_in_bracket_sets_noncompletable() {
        let mut state = bracket_state(BracketState::Empty);
        let result = parse_non_string_data('-', &mut state);
        assert_eq!(result, Ok(Token::NonStringData));
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::NonString(
                NonStringState::NonCompletable("-".to_string())
            )))
        );
    }

    // --- Continue Parsing Tests ---

    #[test]
    fn test_continue_valid_literal() {
        let mut state = brace_state(BraceState::InValue(PrimValue::NonString(
            NonStringState::Completable("t".to_string()),
        )));
        let result = parse_non_string_data('r', &mut state);
        assert_eq!(result, Ok(Token::NonStringData));
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::NonString(
                NonStringState::NonCompletable("tr".to_string())
            )))
        );
        // 'tr' is still incomplete literal; next 'u' then 'e' will flip
        let _ = parse_non_string_data('u', &mut state);
        let _ = parse_non_string_data('e', &mut state);
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::NonString(
                NonStringState::Completable("true".to_string())
            )))
        );
    }

    #[test]
    fn test_continue_valid_number() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::NonString(
            NonStringState::Completable("12".to_string()),
        )));
        let result = parse_non_string_data('3', &mut state);
        assert_eq!(result, Ok(Token::NonStringData));
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::NonString(
                NonStringState::Completable("123".to_string())
            )))
        );
    }

    #[test]
    fn test_number_exponent_incomplete_not_closable_brace() {
        let mut state = brace_state(BraceState::ExpectingValue);
        let _ = parse_non_string_data('1', &mut state);
        let _ = parse_non_string_data('e', &mut state);
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::NonString(
                NonStringState::NonCompletable("1e".to_string())
            )))
        );
    }

    #[test]
    fn test_number_exponent_sign_still_incomplete() {
        let mut state = brace_state(BraceState::ExpectingValue);
        let _ = parse_non_string_data('1', &mut state);
        let _ = parse_non_string_data('e', &mut state);
        let _ = parse_non_string_data('+', &mut state);
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::NonString(
                NonStringState::NonCompletable("1e+".to_string())
            )))
        );
    }

    #[test]
    fn test_number_exponent_becomes_completable_after_digit() {
        let mut state = brace_state(BraceState::ExpectingValue);
        let _ = parse_non_string_data('1', &mut state);
        let _ = parse_non_string_data('e', &mut state);
        let _ = parse_non_string_data('3', &mut state);
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::NonString(
                NonStringState::Completable("1e3".to_string())
            )))
        );
    }

    #[test]
    fn test_continue_to_completion() {
        let mut state = brace_state(BraceState::InValue(PrimValue::NonString(
            NonStringState::Completable("tru".to_string()),
        )));
        let result = parse_non_string_data('e', &mut state);
        assert_eq!(result, Ok(Token::NonStringData));
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::NonString(
                NonStringState::Completable("true".to_string())
            )))
        );
    }

    // --- State Transition to NonCompletable ---

    #[test]
    fn test_continue_invalid_literal_transitions_to_noncompletable() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::NonString(
            NonStringState::Completable("t".to_string()),
        )));
        let result = parse_non_string_data('x', &mut state);
        assert!(result.is_err());
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::NonString(
                NonStringState::NonCompletable("tx".to_string())
            )))
        );
    }

    #[test]
    fn test_continue_invalid_number_transitions_to_noncompletable() {
        let mut state = brace_state(BraceState::InValue(PrimValue::NonString(
            NonStringState::Completable("12".to_string()),
        )));
        let result = parse_non_string_data('a', &mut state);
        assert!(result.is_err());
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::NonString(
                NonStringState::NonCompletable("12a".to_string())
            )))
        );
    }

    // --- Guard Function Tests ---

    #[test]
    fn test_is_non_string_data_guard() {
        // Valid start states
        assert!(is_non_string_data(
            't',
            &brace_state(BraceState::ExpectingValue)
        ));
        assert!(is_non_string_data('1', &bracket_state(BracketState::Empty)));
        assert!(is_non_string_data(
            '-',
            &bracket_state(BracketState::ExpectingValue)
        ));

        // Invalid start states
        assert!(!is_non_string_data(
            't',
            &brace_state(BraceState::ExpectingKey)
        ));
        assert!(!is_non_string_data('1', &JSONState::Pending));

        // Valid continue states
        let continue_state = brace_state(BraceState::InValue(PrimValue::NonString(
            NonStringState::Completable("123".to_string()),
        )));
        assert!(is_non_string_data('4', &continue_state));
        assert!(is_non_string_data('a', &continue_state)); // Guard is permissive, parser is strict
    }
}
