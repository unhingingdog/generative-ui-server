use crate::parser::{get_balancing_chars, modify_stack};
use crate::{lexer, Error};

use super::public_error::Result;
use super::state_types::JSONState;
use super::structural_types::ClosingToken;
use super::structural_types::TokenProcessingError;

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
                Ok(token) => match modify_stack::modify_stack(&mut self.closing_stack, token) {
                    Err(TokenProcessingError::NotAStructuralToken) => {}
                    Err(_) => {
                        self.is_corrupted = true;
                        return Err(Error::Corrupted);
                    }
                    Ok(_) => {}
                },
                Err(e) => {
                    self.is_corrupted = true;
                    return Err(e.into()); // -> Error::Char(CharError)
                }
            }
        }

        Ok(())
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

    // serialize this suite so debug output and states aren't interleaved
    static LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn table_driven_balancing() {
        let _g = LOCK.lock().unwrap();

        for case in CASES {
            let mut bal = JSONBalancer::new();
            let mut last = Ok(String::new());

            for d in case.deltas {
                last = bal.process_delta(d);
                eprintln!("case {} | delta {:?} -> {:?}", case.name, d, last);
            }

            match (&last, &case.outcome) {
                (Ok(s), Outcome::Completion(want)) => {
                    assert_eq!(s, want, "case {}", case.name)
                }
                (Err(e), Outcome::Err(want)) => {
                    assert_eq!(e, want, "case {}", case.name)
                }
                (got, want) => {
                    panic!("case {} mismatch: got {got:?}, want {want:?}", case.name)
                }
            }
        }
    }
}
