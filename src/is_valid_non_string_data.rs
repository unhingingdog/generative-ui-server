use crate::parse_error_types::JSONParseError;

const LITERALS: [&str; 3] = ["true", "false", "null"];

#[derive(Debug, PartialEq)]
pub enum CompletionCheckValues {
    Complete,
    Incomplete,
}

pub fn is_non_valid_non_string_data(
    c: char,
    non_string_data_buffer: &str,
) -> Result<CompletionCheckValues, JSONParseError> {
    let new_value = format!("{}{}", non_string_data_buffer, c);

    let first_char = new_value.chars().next().unwrap_or_default();

    if matches!(first_char, 't' | 'f' | 'n') {
        if LITERALS.contains(&new_value.as_str()) {
            return Ok(CompletionCheckValues::Complete);
        }
        if LITERALS.iter().any(|&lit| lit.starts_with(&new_value)) {
            return Ok(CompletionCheckValues::Incomplete);
        }
        Err(JSONParseError::InvalidCharInLiteral)
    } else if first_char.is_ascii_digit() || first_char == '-' {
        if new_value == "-" {
            return Ok(CompletionCheckValues::Incomplete);
        }

        if new_value.parse::<f64>().is_ok() {
            if new_value.ends_with('.') {
                return Ok(CompletionCheckValues::Incomplete);
            }
            Ok(CompletionCheckValues::Complete)
        } else {
            let last_char = new_value.chars().last().unwrap_or_default();
            if let Some(prefix) = new_value.strip_suffix(last_char) {
                // Example: prefix="123", last_char='e' -> "123e" (Incomplete)
                if prefix.parse::<f64>().is_ok()
                    && (last_char == 'e' || last_char == 'E')
                    && !prefix.contains(['e', 'E'])
                {
                    return Ok(CompletionCheckValues::Incomplete);
                }

                // Example: prefix="1e", last_char='-' -> "1e-" (Incomplete)
                if (prefix.ends_with('e') || prefix.ends_with('E'))
                    && (last_char == '+' || last_char == '-')
                {
                    if let Some(num_part) = prefix.strip_suffix(['e', 'E']) {
                        if num_part.parse::<f64>().is_ok() {
                            return Ok(CompletionCheckValues::Incomplete);
                        }
                    }
                }
            }
            Err(JSONParseError::InvalidCharInNumber)
        }
    } else {
        Err(JSONParseError::InvalidNonStringDataFirstChar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_error_types::JSONParseError;

    fn check(c: char, buffer: &str) -> Result<CompletionCheckValues, JSONParseError> {
        is_non_valid_non_string_data(c, buffer)
    }

    // --- Literal Tests ---

    #[test]
    fn test_literal_incomplete_valid_prefixes() {
        assert_eq!(check('t', ""), Ok(CompletionCheckValues::Incomplete));
        assert_eq!(check('r', "t"), Ok(CompletionCheckValues::Incomplete));
        assert_eq!(check('u', "tr"), Ok(CompletionCheckValues::Incomplete));
        assert_eq!(check('l', "nu"), Ok(CompletionCheckValues::Incomplete));
        assert_eq!(check('s', "fal"), Ok(CompletionCheckValues::Incomplete));
    }

    #[test]
    fn test_literal_complete() {
        assert_eq!(check('e', "tru"), Ok(CompletionCheckValues::Complete));
        assert_eq!(check('l', "nul"), Ok(CompletionCheckValues::Complete));
        assert_eq!(
            check('l', "null"),
            Err(JSONParseError::InvalidCharInLiteral)
        );
        assert_eq!(check('e', "fals"), Ok(CompletionCheckValues::Complete));
    }

    #[test]
    fn test_literal_invalid_prefix() {
        assert_eq!(check('x', "t"), Err(JSONParseError::InvalidCharInLiteral));
        assert_eq!(check('a', "fa"), Err(JSONParseError::InvalidCharInLiteral));
        assert_eq!(
            check('l', "n ull"),
            Err(JSONParseError::InvalidCharInLiteral)
        );
    }

    #[test]
    fn test_literal_too_long() {
        assert_eq!(
            check('x', "true"),
            Err(JSONParseError::InvalidCharInLiteral)
        );
        assert_eq!(
            check('y', "null"),
            Err(JSONParseError::InvalidCharInLiteral)
        );
    }

    // --- Number Tests ---

    #[test]
    fn test_number_complete_integers() {
        assert_eq!(check('1', ""), Ok(CompletionCheckValues::Complete));
        assert_eq!(check('3', "12"), Ok(CompletionCheckValues::Complete));
        assert_eq!(check('9', "-8"), Ok(CompletionCheckValues::Complete));
    }

    #[test]
    fn test_number_complete_floats() {
        assert_eq!(check('5', "123."), Ok(CompletionCheckValues::Complete));
        assert_eq!(check('0', "-0."), Ok(CompletionCheckValues::Complete));
    }

    #[test]
    fn test_number_complete_scientific() {
        assert_eq!(check('5', "1e"), Ok(CompletionCheckValues::Complete));
        assert_eq!(check('2', "1.2e-"), Ok(CompletionCheckValues::Complete));
        assert_eq!(check('9', "-3.14E+1"), Ok(CompletionCheckValues::Complete));
    }

    #[test]
    fn test_number_incomplete_minus_sign() {
        assert_eq!(check('-', ""), Ok(CompletionCheckValues::Incomplete));
    }

    #[test]
    fn test_number_incomplete_decimal() {
        assert_eq!(check('.', "123"), Ok(CompletionCheckValues::Incomplete));
        assert_eq!(check('.', "-0"), Ok(CompletionCheckValues::Incomplete));
    }

    #[test]
    fn test_number_incomplete_exponent() {
        assert_eq!(check('e', "12"), Ok(CompletionCheckValues::Incomplete));
        assert_eq!(check('E', "-7.5"), Ok(CompletionCheckValues::Incomplete));
        assert_eq!(check('-', "1e"), Ok(CompletionCheckValues::Incomplete));
        assert_eq!(check('+', "1.2E"), Ok(CompletionCheckValues::Incomplete));
    }

    #[test]
    fn test_number_invalid() {
        assert_eq!(check('1', "-"), Ok(CompletionCheckValues::Complete)); // "-1" is complete
        assert_eq!(check('.', "123."), Err(JSONParseError::InvalidCharInNumber));
        assert_eq!(check('e', "1e"), Err(JSONParseError::InvalidCharInNumber)); // "1ee" is invalid
        assert_eq!(check('-', "1e-"), Err(JSONParseError::InvalidCharInNumber)); // "1e--" is invalid
        assert_eq!(check('a', "123"), Err(JSONParseError::InvalidCharInNumber));
    }

    // --- Invalid Start Character Tests ---

    #[test]
    fn test_invalid_start_char() {
        assert_eq!(
            check('a', ""),
            Err(JSONParseError::InvalidNonStringDataFirstChar)
        );
        assert_eq!(
            check('_', ""),
            Err(JSONParseError::InvalidNonStringDataFirstChar)
        );
        assert_eq!(
            check('[', ""),
            Err(JSONParseError::InvalidNonStringDataFirstChar)
        );
    }
}
