use crate::lexer::is_non_valid_non_string_data;

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
        use super::state_types::{
            BraceState, BracketState, NonStringState, PrimValue, StringState,
        };

        matches!(
            self,
            JSONState::Pending
                | JSONState::Brace(BraceState::Empty)
                | JSONState::Bracket(BracketState::Empty)
                | JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Closed)))
                | JSONState::Bracket(BracketState::InValue(PrimValue::String(
                    StringState::Closed
                )))
                | JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Open)))
                | JSONState::Bracket(BracketState::InValue(PrimValue::String(StringState::Open)))
                | JSONState::Brace(BraceState::InValue(PrimValue::NonString(
                    NonStringState::Completable(_)
                )))
                | JSONState::Bracket(BracketState::InValue(PrimValue::NonString(
                    NonStringState::Completable(_)
                )))
        )
    }
}

#[cfg(test)]
mod is_cleanly_closable_tests {
    use super::*;

    #[test]
    fn pending_and_empty_containers_are_closable() {
        assert!(JSONState::Pending.is_cleanly_closable());
        assert!(JSONState::Brace(BraceState::Empty).is_cleanly_closable());
        assert!(JSONState::Bracket(BracketState::Empty).is_cleanly_closable());
    }

    #[test]
    fn closed_values_are_closable() {
        assert!(
            JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Closed)))
                .is_cleanly_closable()
        );
        assert!(JSONState::Bracket(BracketState::InValue(PrimValue::String(
            StringState::Closed
        )))
        .is_cleanly_closable());
        assert!(JSONState::Brace(BraceState::InValue(PrimValue::NonString(
            NonStringState::Completable("1".into())
        )))
        .is_cleanly_closable());
        assert!(
            JSONState::Bracket(BracketState::InValue(PrimValue::NonString(
                NonStringState::Completable("1".into())
            )))
            .is_cleanly_closable()
        );
    }

    #[test]
    fn open_string_values_are_closable_by_closing_quote() {
        assert!(
            JSONState::Brace(BraceState::InValue(PrimValue::String(StringState::Open)))
                .is_cleanly_closable()
        );
        assert!(
            JSONState::Bracket(BracketState::InValue(PrimValue::String(StringState::Open)))
                .is_cleanly_closable()
        );
    }

    #[test]
    fn non_completable_nonstring_is_not_closable() {
        assert!(!JSONState::Brace(BraceState::InValue(PrimValue::NonString(
            NonStringState::NonCompletable("1e".into())
        )))
        .is_cleanly_closable());
        assert!(
            !JSONState::Bracket(BracketState::InValue(PrimValue::NonString(
                NonStringState::NonCompletable("1e".into())
            )))
            .is_cleanly_closable()
        );
    }

    #[test]
    fn expecting_key_or_value_is_not_closable() {
        assert!(!JSONState::Brace(BraceState::ExpectingKey).is_cleanly_closable());
        assert!(!JSONState::Brace(BraceState::ExpectingValue).is_cleanly_closable());
        assert!(!JSONState::Bracket(BracketState::ExpectingValue).is_cleanly_closable());
    }
}
