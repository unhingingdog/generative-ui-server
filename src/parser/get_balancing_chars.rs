use crate::JSONState;

use super::structural_types::{BalancingError, ClosingToken};

pub fn get_balancing_chars(
    closing_stack: &[ClosingToken],
    state: &JSONState,
) -> Result<String, BalancingError> {
    if !state.is_cleanly_closable() {
        return Err(BalancingError::NotClosable);
    }

    let closing = closing_stack
        .iter()
        .rev()
        .map(ClosingToken::get_char)
        .collect();

    Ok(closing)
}

#[cfg(test)]
mod tests {
    use crate::parser::state_types::{
        BraceState, BracketState, NonStringState, PrimValue, StringState,
    };

    use super::*;

    // --- Success Cases (Closable States) ---

    #[test]
    fn test_closable_with_empty_stack() {
        let stack = vec![];
        let state = JSONState::Pending; // A valid closable state
        assert_eq!(get_balancing_chars(&stack, &state), Ok("".to_string()));
    }

    #[test]
    fn test_closable_with_single_item_on_stack() {
        let stack = vec![ClosingToken::CloseBrace];
        let state = JSONState::Brace(BraceState::Empty);
        assert_eq!(get_balancing_chars(&stack, &state), Ok("}".to_string()));
    }

    #[test]
    fn test_closable_with_multiple_items_on_stack() {
        // Stack is LIFO, so closing should be in reverse order of this vec.
        let stack = vec![
            ClosingToken::CloseBracket, // Outermost
            ClosingToken::CloseBrace,   // Middle
            ClosingToken::CloseKey,     // Innermost
        ];
        let state = JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Closed)));
        // Should produce "}" followed by "]"
        assert_eq!(get_balancing_chars(&stack, &state), Ok("\"}]".to_string()));
    }

    #[test]
    fn test_all_cleanly_closable_states() {
        let stack = vec![ClosingToken::CloseBrace];
        let closable_states = vec![
            JSONState::Pending,
            JSONState::Brace(BraceState::Empty),
            JSONState::Bracket(BracketState::Empty),
            JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Closed))),
            JSONState::Bracket(BracketState::InValue(PrimValue::String(
                StringState::Closed,
            ))),
            JSONState::Brace(BraceState::InValue(PrimValue::NonString(
                NonStringState::Completable("true".to_string()),
            ))),
            JSONState::Bracket(BracketState::InValue(PrimValue::NonString(
                NonStringState::Completable("123".to_string()),
            ))),
        ];

        for state in closable_states {
            // Using a dummy stack just to confirm it passes the check.
            let result = get_balancing_chars(&stack, &state);
            assert!(
                result.is_ok(),
                "State should have been closable: {:?}",
                state
            );
        }
    }

    // --- Failure Cases (Non-Closable States) ---

    #[test]
    fn test_not_closable_when_expecting_key() {
        let stack = vec![];
        let state = JSONState::Brace(BraceState::ExpectingKey); // Dangling comma
        assert_eq!(
            get_balancing_chars(&stack, &state),
            Err(BalancingError::NotClosable)
        );
    }

    #[test]
    fn test_not_closable_when_in_open_key() {
        let stack = vec![];
        let state = JSONState::Brace(BraceState::InKey(StringState::Open));
        assert_eq!(
            get_balancing_chars(&stack, &state),
            Err(BalancingError::NotClosable)
        );
    }

    #[test]
    fn test_not_closable_when_expecting_value() {
        let stack = vec![];
        let state = JSONState::Brace(BraceState::ExpectingValue); // e.g., after a colon
        assert_eq!(
            get_balancing_chars(&stack, &state),
            Err(BalancingError::NotClosable)
        );
    }

    #[test]
    fn test_not_closable_when_in_open_string_value() {
        let stack = vec![];
        let state = JSONState::Bracket(BracketState::InValue(PrimValue::String(StringState::Open)));
        assert_eq!(
            get_balancing_chars(&stack, &state),
            Err(BalancingError::NotClosable)
        );
    }

    #[test]
    fn test_not_closable_when_non_string_is_non_completable() {
        let stack = vec![];
        let state = JSONState::Brace(BraceState::InValue(PrimValue::NonString(
            NonStringState::NonCompletable("trux".to_string()),
        )));
        assert_eq!(
            get_balancing_chars(&stack, &state),
            Err(BalancingError::NotClosable)
        );
    }
}
