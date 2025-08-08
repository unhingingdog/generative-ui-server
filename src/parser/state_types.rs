#[derive(Debug, PartialEq, Clone)]
pub enum StringState {
    Open,
    Closed,
    Escaped,
}

#[derive(Debug, PartialEq, Clone)]
pub enum NonStringState {
    Completable(String),
    NonCompletable(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrimValue {
    String(StringState),
    NonString(NonStringState),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BraceState {
    Empty,
    ExpectingKey,
    InKey(StringState),
    ExpectingValue,
    InValue(PrimValue),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BracketState {
    Empty,
    InValue(PrimValue),
    ExpectingValue,
}

#[derive(Debug, PartialEq, Clone)]
pub enum JSONState {
    Brace(BraceState),
    Bracket(BracketState),
    Pending,
}

impl JSONState {
    pub fn is_cleanly_closable(&self) -> bool {
        match self {
            // These are the only states where the JSON is "at rest" and can be closed.
            JSONState::Pending // An empty document is fine.
            | JSONState::Brace(BraceState::Empty)
            | JSONState::Bracket(BracketState::Empty) => true,

            // An object is closable if its last element is a fully formed, completable value.
            JSONState::Brace(BraceState::InValue(
                PrimValue::String(StringState::Closed)
                | PrimValue::NonString(NonStringState::Completable(_)),
            )) => true,

            // An array is closable if its last element is a fully formed, completable value.
            JSONState::Bracket(BracketState::InValue(
                PrimValue::String(StringState::Closed)
                | PrimValue::NonString(NonStringState::Completable(_)),
            )) => true,

            // All other states (like `ExpectingKey`, `InKey`, `ExpectingValue`, etc.)
            // are not cleanly closable.
            _ => false,
        }
    }
}
