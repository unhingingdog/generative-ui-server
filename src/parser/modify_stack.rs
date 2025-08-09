use crate::lexer::Token;
use crate::parser::structural_types::{
    ClosingToken, OpeningToken, StructuralToken, TokenProcessingError,
};

pub fn modify_stack(
    stack: &mut Vec<ClosingToken>,
    token: &Token,
) -> Result<(), TokenProcessingError> {
    if let Ok(structural_token) = StructuralToken::try_from(token) {
        if let Ok(opening_token) = OpeningToken::try_from(&structural_token) {
            stack.push(opening_token.get_closing_token());
            return Ok(());
        }
        if let Ok(closing_token) = ClosingToken::try_from(&structural_token) {
            if let Some(current_level_token) = stack.pop() {
                if closing_token == current_level_token {
                    return Ok(());
                } else {
                    stack.push(current_level_token);
                    return Err(TokenProcessingError::CorruptedStackMismatchedTokens);
                }
            } else {
                return Err(TokenProcessingError::CorruptedStackEmptyOnClose);
            }
        }
        return Err(TokenProcessingError::NotAnOpeningOrClosingToken);
    }
    Err(TokenProcessingError::NotAStructuralToken)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token;
    use crate::parser::structural_types::{ClosingToken, TokenProcessingError};

    // --- SUCCESS CASES ---

    #[test]
    fn test_push_open_brace_on_empty_stack() {
        let mut stack = vec![];
        let result = modify_stack(&mut stack, &Token::OpenBrace);
        assert_eq!(result, Ok(()));
        assert_eq!(stack, vec![ClosingToken::CloseBrace]);
    }

    #[test]
    fn test_push_open_key_on_non_empty_stack() {
        let mut stack = vec![ClosingToken::CloseBracket];
        let result = modify_stack(&mut stack, &Token::OpenKey);
        assert_eq!(result, Ok(()));
        assert_eq!(
            stack,
            vec![ClosingToken::CloseBracket, ClosingToken::CloseKey]
        );
    }

    #[test]
    fn test_valid_pop_matching_token() {
        let mut stack = vec![ClosingToken::CloseBrace];
        let result = modify_stack(&mut stack, &Token::CloseBrace);
        assert_eq!(result, Ok(()));
        assert!(stack.is_empty());
    }

    #[test]
    fn test_valid_sequence_push_and_pop() {
        let mut stack = vec![];
        // Simulates processing: `[{`
        modify_stack(&mut stack, &Token::OpenBracket).unwrap();
        modify_stack(&mut stack, &Token::OpenBrace).unwrap();
        assert_eq!(
            stack,
            vec![ClosingToken::CloseBracket, ClosingToken::CloseBrace]
        );

        // Simulates processing: `}]`
        modify_stack(&mut stack, &Token::CloseBrace).unwrap();
        assert_eq!(stack, vec![ClosingToken::CloseBracket]);
        modify_stack(&mut stack, &Token::CloseBracket).unwrap();
        assert!(stack.is_empty());
    }

    // --- ERROR CASES ---

    #[test]
    fn test_err_non_structural_token_comma() {
        let mut stack = vec![];
        let result = modify_stack(&mut stack, &Token::Comma);
        assert_eq!(result, Err(TokenProcessingError::NotAStructuralToken));
        assert!(stack.is_empty()); // Stack should be unchanged
    }

    #[test]
    fn test_err_non_structural_token_whitespace() {
        let mut stack = vec![];
        let result = modify_stack(&mut stack, &Token::Whitespace);
        assert_eq!(result, Err(TokenProcessingError::NotAStructuralToken));
        assert!(stack.is_empty());
    }

    #[test]
    fn test_err_mismatched_closing_token() {
        // Simulates finding a ']' where a '}' was expected.
        let mut stack = vec![ClosingToken::CloseBrace];
        let result = modify_stack(&mut stack, &Token::CloseBracket);
        assert_eq!(
            result,
            Err(TokenProcessingError::CorruptedStackMismatchedTokens)
        );
        // Crucially, the stack should be unchanged after a failed pop attempt.
        assert_eq!(stack, vec![ClosingToken::CloseBrace]);
    }

    #[test]
    fn test_err_closing_token_on_empty_stack() {
        let mut stack = vec![];
        let result = modify_stack(&mut stack, &Token::CloseBracket);
        assert_eq!(
            result,
            Err(TokenProcessingError::CorruptedStackEmptyOnClose)
        );
        assert!(stack.is_empty());
    }

    #[test]
    #[ignore]
    fn test_err_structural_but_not_opening_or_closing() {
        // This test requires a custom setup to simulate the condition.
        // We can't run it directly without modifying the enums, but it documents the case.
        // If `StructuralToken` could contain a variant like `Separator`, this is what would happen:
        // let mut stack = vec![];
        // let token = Token::Separator; // Assume this converts to StructuralToken::Separator
        // let result = modify_stack(&mut stack, &token);
        // assert_eq!(result, Err(TokenProcessingError::NotAnOpeningOrClosingToken));
    }
}
