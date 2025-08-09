use crate::{
    parser::state_types::{BraceState, BracketState, PrimValue, StringState},
    JSONState,
};

use super::{JSONParseError, Token};

#[inline]
fn set_string_state_after_escape_in_place(st: &mut JSONState, next: StringState) -> bool {
    match st {
        JSONState::Brace(BraceState::InKey(StringState::Open)) => {
            *st = JSONState::Brace(BraceState::InKey(next));
            true
        }
        JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Open))) => {
            *st = JSONState::Brace(BraceState::InValue(PrimValue::String(next)));
            true
        }
        JSONState::Bracket(BracketState::InValue(PrimValue::String(StringState::Open))) => {
            *st = JSONState::Bracket(BracketState::InValue(PrimValue::String(next)));
            true
        }
        _ => false,
    }
}

#[inline]
fn set_string_state_from_escaped_in_place(st: &mut JSONState, next: StringState) -> bool {
    match st {
        JSONState::Brace(BraceState::InKey(StringState::Escaped)) => {
            *st = JSONState::Brace(BraceState::InKey(next));
            true
        }
        JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Escaped))) => {
            *st = JSONState::Brace(BraceState::InValue(PrimValue::String(next)));
            true
        }
        JSONState::Bracket(BracketState::InValue(PrimValue::String(StringState::Escaped))) => {
            *st = JSONState::Bracket(BracketState::InValue(PrimValue::String(next)));
            true
        }
        _ => false,
    }
}

/// Called when we read a backslash `\` inside a JSON string (key or value).
/// Transitions String(Open) → String(Escaped).
pub fn handle_escape(current_state: &mut JSONState) -> Result<Token, JSONParseError> {
    if set_string_state_after_escape_in_place(current_state, StringState::Escaped) {
        // IMPORTANT: backslash is not structural; don't push another string frame.
        Ok(Token::StringContent)
    } else {
        Err(JSONParseError::UnexpectedEscape)
    }
}

/// Called for the escaped character that follows a backslash.
/// For standard escapes (`" \ / b f n r t`) we return to Open and emit StringContent.
/// For `\u` we signal "not closable yet" and keep Escaped so the caller won’t close.
pub fn handle_escaped_char(
    escaped: char,
    current_state: &mut JSONState,
) -> Result<Token, JSONParseError> {
    if escaped == 'u' {
        // Stay Escaped; upstream will surface NotClosable
        return Err(JSONParseError::NotClosableInsideUnicode);
    }

    if set_string_state_from_escaped_in_place(current_state, StringState::Open) {
        Ok(Token::StringContent)
    } else {
        Err(JSONParseError::UnexpectedEscape)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::state_types::{BraceState, BracketState, NonStringState, PrimValue};

    fn brace(s: BraceState) -> JSONState {
        JSONState::Brace(s)
    }
    fn bracket(s: BracketState) -> JSONState {
        JSONState::Bracket(s)
    }

    /* ---------- entering escape with '\' ---------- */

    #[test]
    fn escape_in_brace_string_value_enters_escaped() {
        let mut st = brace(BraceState::InValue(PrimValue::String(StringState::Open)));
        let res = handle_escape(&mut st);
        assert_eq!(res, Ok(Token::StringContent));
        assert_eq!(
            st,
            brace(BraceState::InValue(PrimValue::String(StringState::Escaped)))
        );
    }

    #[test]
    fn escape_in_brace_key_enters_escaped() {
        let mut st = brace(BraceState::InKey(StringState::Open));
        let res = handle_escape(&mut st);
        assert_eq!(res, Ok(Token::StringContent));
        assert_eq!(st, brace(BraceState::InKey(StringState::Escaped)));
    }

    #[test]
    fn escape_in_bracket_string_value_enters_escaped() {
        let mut st = bracket(BracketState::InValue(PrimValue::String(StringState::Open)));
        let res = handle_escape(&mut st);
        assert_eq!(res, Ok(Token::StringContent));
        assert_eq!(
            st,
            bracket(BracketState::InValue(PrimValue::String(
                StringState::Escaped
            )))
        );
    }

    #[test]
    fn escape_outside_string_is_error() {
        for mut st in [
            JSONState::Pending,
            brace(BraceState::ExpectingKey),
            brace(BraceState::ExpectingValue),
            brace(BraceState::InValue(PrimValue::NonString(
                NonStringState::Completable("".into()),
            ))),
            bracket(BracketState::ExpectingValue),
            bracket(BracketState::InValue(PrimValue::NonString(
                NonStringState::Completable("".into()),
            ))),
        ] {
            assert_eq!(
                handle_escape(&mut st),
                Err(super::JSONParseError::UnexpectedEscape)
            );
        }
    }

    /* ---------- resolving escaped char ---------- */

    #[test]
    fn escaped_standard_char_returns_to_open_in_key() {
        let mut st = brace(BraceState::InKey(StringState::Escaped));
        let res = handle_escaped_char('n', &mut st);
        assert_eq!(res, Ok(Token::StringContent));
        assert_eq!(st, brace(BraceState::InKey(StringState::Open)));
    }

    #[test]
    fn escaped_standard_char_returns_to_open_in_value_object() {
        let mut st = brace(BraceState::InValue(PrimValue::String(StringState::Escaped)));
        let res = handle_escaped_char('"', &mut st);
        assert_eq!(res, Ok(Token::StringContent));
        assert_eq!(
            st,
            brace(BraceState::InValue(PrimValue::String(StringState::Open)))
        );
    }

    #[test]
    fn escaped_standard_char_returns_to_open_in_value_array() {
        let mut st = bracket(BracketState::InValue(PrimValue::String(
            StringState::Escaped,
        )));
        let res = handle_escaped_char('\\', &mut st);
        assert_eq!(res, Ok(Token::StringContent));
        assert_eq!(
            st,
            bracket(BracketState::InValue(PrimValue::String(StringState::Open)))
        );
    }

    #[test]
    fn escaped_unicode_u_is_not_closable_and_stays_escaped() {
        let mut st = brace(BraceState::InValue(PrimValue::String(StringState::Escaped)));
        let res = handle_escaped_char('u', &mut st);
        assert_eq!(res, Err(JSONParseError::NotClosableInsideUnicode));
        assert_eq!(
            st,
            brace(BraceState::InValue(PrimValue::String(StringState::Escaped)))
        );
    }

    #[test]
    fn escaped_char_called_when_not_in_escaped_is_error() {
        for mut st in [
            JSONState::Pending,
            brace(BraceState::InKey(StringState::Open)),
            brace(BraceState::InValue(PrimValue::String(StringState::Open))),
            bracket(BracketState::InValue(PrimValue::String(StringState::Open))),
        ] {
            assert_eq!(
                handle_escaped_char('n', &mut st),
                Err(super::JSONParseError::UnexpectedEscape)
            );
        }
    }
}
