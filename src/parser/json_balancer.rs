use crate::lexer;
use crate::parser::{get_balancing_chars, modify_stack};

use super::state_types::JSONState;
use super::structural_types::ClosingToken;
use super::structural_types::{BalancingError, TokenProcessingError};

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

    pub fn process_delta(&mut self, delta: &str) {
        self.get_debug_state(delta, "start", None);
        // If the stream is already corrupted, do nothing.
        if self.is_corrupted {
            return;
        }

        for c in delta.chars() {
            self.get_debug_state(delta, "before first char", Some(c));
            match lexer::parse_char(c, &mut self.state) {
                Ok(token) => {
                    // This is the existing logic for a successful token parse.
                    match modify_stack::modify_stack(&mut self.closing_stack, token) {
                        Err(TokenProcessingError::NotAStructuralToken) => (),
                        Err(e) => {
                            self.get_debug_state(delta, "start", Some(c));
                            self.is_corrupted = true;
                            println!("ERROR - token processing: {:?}", e);
                            break;
                        }
                        Ok(_) => (),
                    }
                }
                Err(e) => {
                    // This now captures and logs the specific error from parse_char.
                    println!("ERROR - hard lexical error: {:?}", e);
                    self.is_corrupted = true;
                    break;
                }
            }
        }
    }

    /// Returns the string of characters required to validly close the JSON
    /// structure based on the current state.
    ///
    /// This method takes `&self` because it only needs to read the state.
    /// It returns an error if the JSON is in a state that cannot be cleanly closed.
    pub fn get_completion(&self) -> Result<String, BalancingError> {
        // First, check for an unrecoverable corrupted state.
        if self.is_corrupted {
            return Err(BalancingError::Corrupted);
        }

        // Then, delegate to the existing logic which handles temporarily un-closable states.
        get_balancing_chars::get_balancing_chars(&self.closing_stack, &self.state)
    }
}

/// Implementing Default is idiomatic for structs that have a clear "empty" state.
/// It allows users to create a new instance with `JSONBalancer::default()`.
impl Default for JSONBalancer {
    fn default() -> Self {
        JSONBalancer {
            closing_stack: Vec::new(),
            state: JSONState::Pending,
            is_corrupted: false, // Start in a valid state
        }
    }
}

// --- Example Usage ---
// This shows how a consumer of your library (e.g., your WASM module) would use it.
#[cfg(test)]
mod tests {
    use super::*;
    // Assume BalancingError is updated to include a `Corrupted` variant
    use crate::parser::structural_types::BalancingError;

    #[test]
    fn test_streaming_usage() {
        // 1. Initialize the balancer.
        let mut balancer = JSONBalancer::new();
        let mut full_json = String::new();

        // 2. First delta comes in.
        let delta1 = "[{\"key\":";
        full_json.push_str(delta1);
        balancer.process_delta(delta1);
        // At this point, the state is ExpectingValue, which is not closable but is not corrupted.
        assert_eq!(balancer.get_completion(), Err(BalancingError::NotClosable));

        // 3. Second delta comes in.
        let delta2 = "\"value\"";
        full_json.push_str(delta2);
        balancer.process_delta(delta2);
        let completion2 = balancer.get_completion().unwrap_or_default();
        // Now the state is valid and closable.
        assert_eq!(completion2, "}]");
        println!("After delta 2, display: {}{}", full_json, completion2);

        // 4. A corrupted delta comes in.
        let delta3 = "]"; // Mismatched closer
        full_json.push_str(delta3);
        balancer.process_delta(delta3);
        // The balancer should now be in a permanently corrupted state.
        assert_eq!(balancer.get_completion(), Err(BalancingError::Corrupted));

        // 5. Any subsequent delta is ignored.
        let delta4 = "{\"another\": 1}";
        balancer.process_delta(delta4);
        assert_eq!(balancer.get_completion(), Err(BalancingError::Corrupted));
        println!("Final state is corrupted, as expected.");
    }
}
