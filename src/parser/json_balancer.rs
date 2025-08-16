use crate::lexer::{JSONParseError, Token};
use crate::parser::{get_balancing_chars, modify_stack};
use crate::{lexer, Error};

use super::public_error::Result;
use super::state_types::{BraceState, BracketState, JSONState, NonStringState, PrimValue};
use super::structural_types::TokenProcessingError;
use super::structural_types::{ClosingToken, PopLevelToken};

pub struct JSONBalancer {
    closing_stack: Vec<ClosingToken>,
    state: JSONState,
    is_corrupted: bool,
}

impl JSONBalancer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process_delta(&mut self, delta: &str) -> Result<String> {
        self.add_delta(delta)?;
        self.get_completion()
    }

    fn add_delta(&mut self, delta: &str) -> Result<()> {
        if self.is_corrupted {
            return Err(Error::Corrupted);
        }

        for c in delta.chars() {
            match lexer::parse_char(c, &mut self.state) {
                Ok(token) => match modify_stack::modify_stack(&mut self.closing_stack, &token) {
                    Ok(_) => self.handle_pop_state_transition(token),
                    Err(
                        TokenProcessingError::NotAStructuralToken
                        | TokenProcessingError::NotAnOpeningOrClosingToken,
                    ) => {}
                    Err(_) => {
                        self.is_corrupted = true;
                        return Err(Error::Corrupted);
                    }
                },
                Err(e) => {
                    if matches!(e, JSONParseError::NotClosableInsideUnicode) {
                        // This is a hack around the fact we have no NonStringData InUnicode substate (for now).
                        // This is a "soft" error. We return NotClosable and do NOT corrupt the stream.
                        return Err(Error::NotClosable);
                    } else {
                        // This is a "hard" lexer error. We corrupt the stream and return the specific error.
                        self.is_corrupted = true;
                        return Err(e.into());
                    }
                }
            }
        }
        Ok(())
    }

    // We need this to get back to the reverse-recursive parent state.
    fn handle_pop_state_transition(&mut self, token: Token) {
        if PopLevelToken::try_from(&token).is_ok() {
            self.state = match self.closing_stack.last() {
                // The parent is an object. We just completed a value within it.
                Some(ClosingToken::CloseBrace) => {
                    JSONState::Brace(BraceState::InValue(PrimValue::NestedValueCompleted))
                }
                // The parent is an array. We just completed a value within it.
                Some(ClosingToken::CloseBracket) => {
                    JSONState::Bracket(BracketState::InValue(PrimValue::NestedValueCompleted))
                }
                // The stack is now empty; the entire document is closed.
                None => JSONState::Pending,
                // The parent is a string (e.g., we just closed a key). The state
                // is already handled by the lexer, so we don't need to do anything here.
                _ => return,
            };
        }
    }

    fn get_completion(&self) -> Result<String> {
        if self.is_corrupted {
            return Err(Error::Corrupted);
        }
        get_balancing_chars::get_balancing_chars(&self.closing_stack, &self.state)
            .map_err(Into::into)
    }
}

impl Default for JSONBalancer {
    fn default() -> Self {
        JSONBalancer {
            closing_stack: Vec::new(),
            state: JSONState::Pending,
            is_corrupted: false, // Start in a valid state
        }
    }
}

#[cfg(test)]
mod pop_state_tests {
    use super::super::structural_types::ClosingToken::*;
    use super::*;
    use crate::parser::state_types::*;

    #[test]
    fn pop_after_close_brace_parent_is_brace() {
        let mut b = JSONBalancer::new();
        b.closing_stack = vec![CloseBrace];
        b.state = JSONState::Brace(BraceState::ExpectingKey);
        b.handle_pop_state_transition(Token::CloseBrace);
        assert!(matches!(
            b.state,
            JSONState::Brace(BraceState::InValue(PrimValue::NestedValueCompleted))
        ));
    }

    #[test]
    fn pop_after_close_brace_parent_is_bracket() {
        let mut b = JSONBalancer::new();
        b.closing_stack = vec![CloseBracket];
        b.state = JSONState::Bracket(BracketState::ExpectingValue);
        b.handle_pop_state_transition(Token::CloseBrace);
        assert!(matches!(
            b.state,
            JSONState::Bracket(BracketState::InValue(PrimValue::NestedValueCompleted))
        ));
    }

    #[test]
    fn pop_after_close_bracket_parent_is_brace() {
        let mut b = JSONBalancer::new();
        b.closing_stack = vec![CloseBrace];
        b.state = JSONState::Brace(BraceState::ExpectingValue);
        b.handle_pop_state_transition(Token::CloseBracket);
        assert!(matches!(
            b.state,
            JSONState::Brace(BraceState::InValue(PrimValue::NestedValueCompleted))
        ));
    }

    #[test]
    fn pop_to_pending_when_stack_empty() {
        let mut b = JSONBalancer::new();
        b.closing_stack.clear();
        b.state = JSONState::Brace(BraceState::Empty);
        b.handle_pop_state_transition(Token::CloseBrace);
        assert!(matches!(b.state, JSONState::Pending));
        b.state = JSONState::Bracket(BracketState::Empty);
        b.handle_pop_state_transition(Token::CloseBracket);
        assert!(matches!(b.state, JSONState::Pending));
    }

    #[test]
    fn non_pop_token_no_change() {
        let mut b = JSONBalancer::new();
        b.closing_stack = vec![CloseBrace];
        b.state = JSONState::Brace(BraceState::ExpectingKey);
        b.handle_pop_state_transition(Token::Comma);
        assert!(matches!(
            b.state,
            JSONState::Brace(BraceState::ExpectingKey)
        ));
    }
}
