use crate::JSONState;

use super::structural_types::ClosingToken;

pub struct JSONBalancer {
    closing_stack: Vec<ClosingToken>,
    state: JSONState,
}

impl JSONBalancer {
    pub fn new() -> Self {
        JSONBalancer {
            closing_stack: Vec::new(),
            state: JSONState::Pending,
        }
    }
}

impl Default for JSONBalancer {
    fn default() -> Self {
        JSONBalancer {
            closing_stack: Vec::new(),
            state: JSONState::Pending,
        }
    }
}
