use crate::lexer::Token;
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

    pub fn get_debug_state(&self, delta: &str, tag: &str, c: Option<char>) {
        println!("-----{}---------", tag);
        println!("delta: {}", delta);
        println!("char: {}", c.unwrap_or('N'));
        println!("state {:?}", self.state);
        println!("stack {:?}", self.closing_stack);
        println!("is corrupted? {:?}", self.is_corrupted);
        println!("-----end---------");
    }

    pub fn process_delta(&mut self, delta: &str) -> Result<String> {
        self.add_delta(delta)?;
        self.get_completion()
    }

    fn add_delta(&mut self, delta: &str) -> Result<()> {
        self.get_debug_state(delta, "start", None);

        if self.is_corrupted {
            return Err(Error::Corrupted);
        }

        for c in delta.chars() {
            self.get_debug_state(delta, "before first char", Some(c));
            match lexer::parse_char(c, &mut self.state) {
                Ok(token) => match modify_stack::modify_stack(&mut self.closing_stack, &token) {
                    Err(TokenProcessingError::NotAStructuralToken) => {}
                    Err(_) => {
                        self.is_corrupted = true;
                        return Err(Error::Corrupted);
                    }
                    Ok(_) => self.handle_pop_state_transition(token),
                },
                Err(_) => {
                    self.is_corrupted = true;
                    return Err(Error::Corrupted);
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
                Some(ClosingToken::CloseBrace) => JSONState::Brace(BraceState::InValue(
                    PrimValue::NonString(NonStringState::Completable(String::new())),
                )),
                // The parent is an array. We just completed a value within it.
                Some(ClosingToken::CloseBracket) => JSONState::Bracket(BracketState::InValue(
                    PrimValue::NonString(NonStringState::Completable(String::new())),
                )),
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
mod tests {
    use super::*;
    use crate::parser::balancing_test_data::{Outcome, CASES};
    use std::sync::Mutex;

    // A lock to serialize tests, ensuring debug output is not interleaved (comment out when not
    // debugging).
    static LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn table_driven_balancing() {
        let _g = LOCK.lock().unwrap();

        for case in CASES {
            let mut bal = JSONBalancer::new();
            let mut last = Ok(String::new());

            for d in case.deltas {
                last = bal.process_delta(d);
            }

            // This is the updated assertion logic. It provides a much clearer
            // message on failure, including the case name, deltas, expected
            // outcome, and the actual outcome.
            let (success, expected_str, actual_str) = match (&last, &case.outcome) {
                (Ok(s), Outcome::Completion(want)) => (
                    s.as_str() == *want,
                    format!("Completion(\"{}\")", want),
                    format!("Ok(\"{}\")", s),
                ),
                (Err(e), Outcome::Err(want)) => (
                    e == want,
                    format!("Err({:?})", want),
                    format!("Err({:?})", e),
                ),
                (got, want) => (false, format!("{:?}", want), format!("{:?}", got)),
            };

            if !success {
                panic!(
                    "\n\nAssertion failed for test case: '{}'\n  Deltas:   {:?}\n  Expected: {}\n  Got:      {}\n\n",
                    case.name, case.deltas, expected_str, actual_str
                );
            }
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
        b.closing_stack = vec![CloseBrace]; // after popping inner '}', parent is an object
        b.state = JSONState::Brace(BraceState::ExpectingKey);
        b.handle_pop_state_transition(Token::CloseBrace);
        assert!(matches!(
            b.state,
            JSONState::Brace(BraceState::InValue(PrimValue::NonString(
                NonStringState::Completable(_)
            )))
        ));
    }

    #[test]
    fn pop_after_close_brace_parent_is_bracket() {
        let mut b = JSONBalancer::new();
        b.closing_stack = vec![CloseBracket]; // object closed inside an array
        b.state = JSONState::Bracket(BracketState::ExpectingValue);
        b.handle_pop_state_transition(Token::CloseBrace);
        assert!(matches!(
            b.state,
            JSONState::Bracket(BracketState::InValue(PrimValue::NonString(
                NonStringState::Completable(_)
            )))
        ));
    }

    #[test]
    fn pop_after_close_bracket_parent_is_brace() {
        let mut b = JSONBalancer::new();
        b.closing_stack = vec![CloseBrace]; // array closed inside an object
        b.state = JSONState::Brace(BraceState::ExpectingValue);
        b.handle_pop_state_transition(Token::CloseBracket);
        assert!(matches!(
            b.state,
            JSONState::Brace(BraceState::InValue(PrimValue::NonString(
                NonStringState::Completable(_)
            )))
        ));
    }

    #[test]
    fn pop_to_pending_when_stack_empty() {
        let mut b = JSONBalancer::new();
        b.closing_stack.clear(); // top-level container just closed
        b.state = JSONState::Brace(BraceState::Empty);
        b.handle_pop_state_transition(Token::CloseBrace);
        assert!(matches!(b.state, JSONState::Pending));
        // also verify with a bracket close
        b.state = JSONState::Bracket(BracketState::Empty);
        b.handle_pop_state_transition(Token::CloseBracket);
        assert!(matches!(b.state, JSONState::Pending));
    }

    #[test]
    fn non_pop_token_no_change() {
        let mut b = JSONBalancer::new();
        b.closing_stack = vec![CloseBrace];
        b.state = JSONState::Brace(BraceState::ExpectingKey);
        b.handle_pop_state_transition(Token::Comma); // not a PopLevelToken
        assert!(matches!(
            b.state,
            JSONState::Brace(BraceState::ExpectingKey)
        ));
    }
}
