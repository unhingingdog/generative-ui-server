use crate::parse_error_types::JSONParseError;
use crate::state_types::*;

fn is_non_string_start(c: char) -> bool {
    c.is_ascii_digit() || c == '-' || matches!(c, 'n' | 't' | 'f')
}

fn is_non_string_char(c: char) -> bool {
    c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-') || c.is_ascii_alphabetic()
}

pub fn is_non_string_data(c: char, state: &JSONState) -> bool {
    match state {
        JSONState::Brace(BraceState::ExpectingValue)
        | JSONState::Bracket(BracketState::ExpectingValue) => is_non_string_start(c),

        JSONState::Brace(BraceState::InValue(PrimValue::NonString))
        | JSONState::Bracket(BracketState::InValue(PrimValue::NonString)) => is_non_string_char(c),

        _ => false,
    }
}

pub fn parse_non_string_data(state: &mut JSONState) -> Result<Token, JSONParseError> {
    match state {
        JSONState::Brace(bs) => match bs {
            BraceState::ExpectingValue => {
                *bs = BraceState::InValue(PrimValue::NonString);
                Ok(Token::NonStringData)
            }
            BraceState::InValue(PrimValue::NonString) => Ok(Token::NonStringData),
            _ => Err(JSONParseError::UnexpectedCharInNonStringData),
        },
        JSONState::Bracket(bs) => match bs {
            BracketState::ExpectingValue => {
                *bs = BracketState::InValue(PrimValue::NonString);
                Ok(Token::NonStringData)
            }
            BracketState::InValue(PrimValue::NonString) => Ok(Token::NonStringData),
            _ => Err(JSONParseError::UnexpectedCharInNonStringData),
        },
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

    // Tests for bracket state transitions
    #[test]
    fn test_bracket_move_from_expecting_to_non_string_data() {
        let mut state = bracket_state(BracketState::ExpectingValue);
        let result = parse_non_string_data(&mut state);

        assert_eq!(result.unwrap(), Token::NonStringData);
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::NonString))
        );
    }

    #[test]
    fn test_bracket_stay_in_non_string_data() {
        let mut state = bracket_state(BracketState::InValue(PrimValue::NonString));
        let result = parse_non_string_data(&mut state);

        assert_eq!(result.unwrap(), Token::NonStringData);
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::NonString))
        );
    }

    // Tests for brace state transitions
    #[test]
    fn test_brace_move_from_expecting_to_non_string_data() {
        let mut state = brace_state(BraceState::ExpectingValue);
        let result = parse_non_string_data(&mut state);

        assert_eq!(result.unwrap(), Token::NonStringData);
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::NonString))
        );
    }

    #[test]
    fn test_brace_stay_in_non_string_data() {
        let mut state = brace_state(BraceState::InValue(PrimValue::NonString));
        let result = parse_non_string_data(&mut state);

        assert_eq!(result.unwrap(), Token::NonStringData);
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::NonString))
        );
    }

    #[test]
    fn test_bracket_error_from_invalid_state() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Open)));
        let result = parse_non_string_data(&mut state);

        assert_eq!(result, Err(JSONParseError::UnexpectedCharInNonStringData));
    }

    // Error case tests for brace state
    #[test]
    fn test_brace_error_from_invalid_state() {
        let mut state = brace_state(BraceState::InValue(PrimValue::String(StringState::Open)));
        let result = parse_non_string_data(&mut state);

        assert_eq!(result, Err(JSONParseError::UnexpectedCharInNonStringData));
    }

    #[test]
    fn test_brace_error_from_expecting_key() {
        let mut state = brace_state(BraceState::ExpectingKey);
        let result = parse_non_string_data(&mut state);

        assert_eq!(result, Err(JSONParseError::UnexpectedCharInNonStringData));
    }

    // Edge case tests
    #[test]
    fn test_multiple_calls_bracket_expecting_value() {
        let mut state = bracket_state(BracketState::ExpectingValue);

        // First call should transition to InValue(NonString)
        let result1 = parse_non_string_data(&mut state);
        assert_eq!(result1, Ok(Token::NonStringData));
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::NonString))
        );

        // Second call should stay in InValue(NonString)
        let result2 = parse_non_string_data(&mut state);
        assert_eq!(result2, Ok(Token::NonStringData));
        assert_eq!(
            state,
            bracket_state(BracketState::InValue(PrimValue::NonString))
        );
    }

    #[test]
    fn test_multiple_calls_brace_expecting_value() {
        let mut state = brace_state(BraceState::ExpectingValue);

        // First call should transition to InValue(NonString)
        let result1 = parse_non_string_data(&mut state);
        assert_eq!(result1, Ok(Token::NonStringData));
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::NonString))
        );

        // Second call should stay in InValue(NonString)
        let result2 = parse_non_string_data(&mut state);
        assert_eq!(result2, Ok(Token::NonStringData));
        assert_eq!(
            state,
            brace_state(BraceState::InValue(PrimValue::NonString))
        );
    }

    mod is_non_string_start_tests {
        use super::*;

        #[test]
        fn test_digits() {
            for c in '0'..='9' {
                assert!(
                    is_non_string_start(c),
                    "Digit '{}' should be non-string start",
                    c
                );
            }
        }

        #[test]
        fn test_minus_sign() {
            assert!(is_non_string_start('-'));
        }

        #[test]
        fn test_null_true_false_chars() {
            assert!(is_non_string_start('n')); // null
            assert!(is_non_string_start('t')); // true
            assert!(is_non_string_start('f')); // false
        }

        #[test]
        fn test_other_letters() {
            let letters = [
                'a', 'b', 'c', 'd', 'e', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'o', 'p', 'q', 'r',
                's', 'u', 'v', 'w', 'x', 'y', 'z',
            ];
            for &c in &letters {
                assert!(
                    !is_non_string_start(c),
                    "Letter '{}' should not be non-string start",
                    c
                );
            }
        }

        #[test]
        fn test_uppercase_letters() {
            let letters = ['A', 'B', 'C', 'N', 'T', 'F', 'X', 'Y', 'Z'];
            for &c in &letters {
                assert!(
                    !is_non_string_start(c),
                    "Uppercase '{}' should not be non-string start",
                    c
                );
            }
        }

        #[test]
        fn test_special_chars() {
            let chars = [
                '+', '.', 'e', 'E', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '[', ']',
                '{', '}', '|', '\\', ':', ';', '"', '\'', '<', '>', ',', '?', '/', '~', '`',
            ];
            for &c in &chars {
                assert!(
                    !is_non_string_start(c),
                    "Special char '{}' should not be non-string start",
                    c
                );
            }
        }

        #[test]
        fn test_whitespace() {
            assert!(!is_non_string_start(' '));
            assert!(!is_non_string_start('\t'));
            assert!(!is_non_string_start('\n'));
            assert!(!is_non_string_start('\r'));
        }
    }

    // Tests for is_non_string_char
    mod is_non_string_char_tests {
        use super::*;

        #[test]
        fn test_digits() {
            for c in '0'..='9' {
                assert!(
                    is_non_string_char(c),
                    "Digit '{}' should be non-string char",
                    c
                );
            }
        }

        #[test]
        fn test_number_special_chars() {
            assert!(is_non_string_char('.')); // decimal point
            assert!(is_non_string_char('e')); // scientific notation
            assert!(is_non_string_char('E')); // scientific notation
            assert!(is_non_string_char('+')); // positive exponent
            assert!(is_non_string_char('-')); // negative number/exponent
        }

        #[test]
        fn test_ascii_alphabetic() {
            // Test lowercase
            for c in 'a'..='z' {
                assert!(
                    is_non_string_char(c),
                    "Lowercase '{}' should be non-string char",
                    c
                );
            }
            // Test uppercase
            for c in 'A'..='Z' {
                assert!(
                    is_non_string_char(c),
                    "Uppercase '{}' should be non-string char",
                    c
                );
            }
        }

        #[test]
        fn test_non_ascii_chars() {
            let chars = ['ñ', 'ü', 'ç', '中', '🙂'];
            for &c in &chars {
                assert!(
                    !is_non_string_char(c),
                    "Non-ASCII '{}' should not be non-string char",
                    c
                );
            }
        }

        #[test]
        fn test_other_special_chars() {
            let chars = [
                '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '[', ']', '{', '}', '|', '\\',
                ':', ';', '"', '\'', '<', '>', ',', '?', '/', '~', '`',
            ];
            for &c in &chars {
                assert!(
                    !is_non_string_char(c),
                    "Special char '{}' should not be non-string char",
                    c
                );
            }
        }

        #[test]
        fn test_whitespace() {
            assert!(!is_non_string_char(' '));
            assert!(!is_non_string_char('\t'));
            assert!(!is_non_string_char('\n'));
            assert!(!is_non_string_char('\r'));
        }
    }

    mod is_non_string_data_tests {
        use super::*;

        #[test]
        fn test_expecting_value_states_use_start_logic() {
            let states = [
                brace_state(BraceState::ExpectingValue),
                bracket_state(BracketState::ExpectingValue),
            ];

            for state in &states {
                // Should return true for valid start chars
                assert!(is_non_string_data('5', state));
                assert!(is_non_string_data('-', state));
                assert!(is_non_string_data('n', state));
                assert!(is_non_string_data('t', state));
                assert!(is_non_string_data('f', state));

                // Should return false for invalid start chars
                assert!(!is_non_string_data('a', state));
                assert!(!is_non_string_data('.', state));
                assert!(!is_non_string_data('e', state));
                assert!(!is_non_string_data('+', state));
            }
        }

        #[test]
        fn test_in_value_states_use_char_logic() {
            let states = [
                brace_state(BraceState::InValue(PrimValue::NonString)),
                bracket_state(BracketState::InValue(PrimValue::NonString)),
            ];

            for state in &states {
                // Should return true for valid continuation chars
                assert!(is_non_string_data('5', state));
                assert!(is_non_string_data('.', state));
                assert!(is_non_string_data('e', state));
                assert!(is_non_string_data('E', state));
                assert!(is_non_string_data('+', state));
                assert!(is_non_string_data('-', state));
                assert!(is_non_string_data('a', state)); // alphabetic
                assert!(is_non_string_data('Z', state)); // alphabetic

                // Should return false for invalid chars
                assert!(!is_non_string_data('!', state));
                assert!(!is_non_string_data(' ', state));
                assert!(!is_non_string_data(',', state));
            }
        }

        #[test]
        fn test_other_states_return_false() {
            // Test with states that should always return false
            let states = [
                brace_state(BraceState::InValue(PrimValue::String(StringState::Open))), // Assuming String variant exists
                bracket_state(BracketState::InValue(PrimValue::String(StringState::Open))),
                // Add other states as needed based on your actual enum variants
            ];

            for state in &states {
                // Should return false for any character
                assert!(!is_non_string_data('5', state));
                assert!(!is_non_string_data('-', state));
                assert!(!is_non_string_data('n', state));
                assert!(!is_non_string_data('a', state));
                assert!(!is_non_string_data('.', state));
                assert!(!is_non_string_data(' ', state));
            }
        }

        #[test]
        fn test_realistic_number_parsing() {
            // Test realistic number parsing scenarios
            let expecting_state = bracket_state(BracketState::ExpectingValue);
            let in_value_state = bracket_state(BracketState::InValue(PrimValue::NonString));

            // Start of number
            assert!(is_non_string_data('1', &expecting_state));
            assert!(is_non_string_data('-', &expecting_state));

            // Continuation of number
            assert!(is_non_string_data('2', &in_value_state));
            assert!(is_non_string_data('.', &in_value_state));
            assert!(is_non_string_data('5', &in_value_state));
            assert!(is_non_string_data('e', &in_value_state));
            assert!(is_non_string_data('-', &in_value_state));
            assert!(is_non_string_data('1', &in_value_state));
            assert!(is_non_string_data('0', &in_value_state));
        }

        #[test]
        fn test_realistic_boolean_null_parsing() {
            let expecting_state = brace_state(BraceState::ExpectingValue);
            let in_value_state = brace_state(BraceState::InValue(PrimValue::NonString));

            // Start of literals
            assert!(is_non_string_data('t', &expecting_state)); // true
            assert!(is_non_string_data('f', &expecting_state)); // false
            assert!(is_non_string_data('n', &expecting_state)); // null

            // Continuation of literals
            assert!(is_non_string_data('r', &in_value_state)); // t[r]ue
            assert!(is_non_string_data('u', &in_value_state)); // tr[u]e
            assert!(is_non_string_data('e', &in_value_state)); // tru[e]
            assert!(is_non_string_data('a', &in_value_state)); // f[a]lse
            assert!(is_non_string_data('l', &in_value_state)); // fa[l]se
        }
    }
}
