#![cfg(test)]

use crate::Error;

#[derive(Debug)]
pub enum Outcome {
    Completion(&'static str),
    Err(Error),
}

#[derive(Debug)]
pub struct Case {
    pub name: &'static str,
    pub deltas: &'static [&'static str],
    pub outcome: Outcome,
}

pub const EMPTY_ARRAY: Case = Case {
    name: "empty_array",
    deltas: &["["],
    outcome: Outcome::Completion("]"),
};

pub const EMPTY_OBJECT: Case = Case {
    name: "empty_object",
    deltas: &["{"],
    outcome: Outcome::Completion("}"),
};

pub const ARRAY_ONE_NUMBER: Case = Case {
    name: "array_one_number",
    deltas: &["[", "1"],
    outcome: Outcome::Completion("]"),
};

pub const OBJECT_SIMPLE_KV: Case = Case {
    name: "object_simple_kv",
    deltas: &["{", r#""a""#, ":", "1"],
    outcome: Outcome::Completion("}"),
};

pub const NESTED_OBJECT_IN_ARRAY: Case = Case {
    name: "nested_object_in_array",
    deltas: &["[", "{", r#""k""#, ":", r#""v""#],
    outcome: Outcome::Completion("}]"),
};

pub const DOUBLE_NEST: Case = Case {
    name: "double_nest",
    deltas: &["{", r#""a""#, ":", "[", "{", r#""b""#, ":", "2"],
    outcome: Outcome::Completion("}]}"),
};

pub const MULTI_KV_OBJECT_NEEDS_BRACE: Case = Case {
    name: "multi_kv_object_needs_brace",
    deltas: &["{", r#""a""#, ":", "1", ",", r#""b""#, ":", "2"],
    outcome: Outcome::Completion("}"),
};

pub const TRAILING_STRING_VALUE: Case = Case {
    name: "trailing_string_value",
    deltas: &["{", r#""a""#, ":", r#""x""#],
    outcome: Outcome::Completion("}"),
};

pub const ARRAY_OF_OBJECTS_PARTIAL_SECOND: Case = Case {
    name: "array_of_objects_partial_second",
    deltas: &[
        "[", "{", r#""a""#, ":", "1", "}", ",", "{", r#""b""#, ":", "2",
    ],
    outcome: Outcome::Completion("}]"),
};

// ----- Not closable yet (valid so far, but need more chars) -----
pub const OBJ_EXPECTING_COLON: Case = Case {
    name: "obj_expecting_colon",
    deltas: &["{", r#""a""#],
    outcome: Outcome::Err(Error::NotClosable),
};

pub const OBJ_EXPECTING_VALUE: Case = Case {
    name: "obj_expecting_value",
    deltas: &["{", r#""a""#, ":"],
    outcome: Outcome::Err(Error::NotClosable),
};

pub const OBJ_IN_OPEN_STRING_VALUE: Case = Case {
    name: "obj_in_open_string_value",
    deltas: &["{", r#""a""#, ":", r#""va"#],
    outcome: Outcome::Completion("\"}"),
};

pub const OBJ_IN_ESCAPE: Case = Case {
    name: "obj_in_escape",
    deltas: &["{", r#""a""#, ":", r#""va\"#], // backslash before the closing quote
    outcome: Outcome::Err(Error::NotClosable),
};

pub const ARRAY_AFTER_COMMA_EXPECTING_VALUE: Case = Case {
    name: "array_after_comma_expecting_value",
    deltas: &["[", "1", ",", ""],
    outcome: Outcome::Err(Error::NotClosable),
};

pub const ARRAY_IN_OPEN_STRING: Case = Case {
    name: "array_in_open_string",
    deltas: &["[", r#""hel"#],
    outcome: Outcome::Completion("\"]"),
};

pub const NUMBER_PARTIAL_MINUS: Case = Case {
    name: "number_partial_minus",
    deltas: &["{", r#""n""#, ":", "-"],
    outcome: Outcome::Err(Error::NotClosable),
};

pub const NUMBER_PARTIAL_EXP: Case = Case {
    name: "number_partial_exp",
    deltas: &["{", r#""n""#, ":", "1e"],
    outcome: Outcome::Err(Error::NotClosable),
};

pub const LITERAL_TRUE_PARTIAL: Case = Case {
    name: "literal_true_partial",
    deltas: &["[", "tr"],
    outcome: Outcome::Err(Error::NotClosable),
};

pub const LITERAL_NULL_PARTIAL: Case = Case {
    name: "literal_null_partial",
    deltas: &["{", r#""x""#, ":", "nu"],
    outcome: Outcome::Err(Error::NotClosable),
};

// ----- Corrupted states (irrecoverable) -----
pub const CORRUPTED_MISMATCH: Case = Case {
    name: "corrupted_mismatch",
    deltas: &["[", "]", "]"],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const CORRUPTED_EXTRA_COLON: Case = Case {
    name: "corrupted_extra_colon",
    deltas: &["{", r#""a""#, ":", ":", "1"],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const CORRUPTED_CLOSE_BRACE_IN_ARRAY: Case = Case {
    name: "corrupted_close_brace_in_array",
    deltas: &["[", "}"],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const CORRUPTED_UNEXPECTED_COMMA_START_ARRAY: Case = Case {
    name: "corrupted_unexpected_comma_start_array",
    deltas: &["[", ","],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const CORRUPTED_UNEXPECTED_COMMA_START_OBJECT: Case = Case {
    name: "corrupted_unexpected_comma_start_object",
    deltas: &["{", ","],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const CORRUPTED_UNEXPECTED_COLON_TOP: Case = Case {
    name: "corrupted_unexpected_colon_top",
    deltas: &[":"],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const CORRUPTED_QUOTE_IN_NONSTRING_DATA: Case = Case {
    name: "corrupted_quote_in_nonstring_data",
    deltas: &["[", "1", "\"", "]"],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const CORRUPTED_CLOSE_BEFORE_KEY: Case = Case {
    name: "corrupted_close_before_key",
    deltas: &["{", "}"], // immediately closing is fine actually; make it bad by adding comma then brace
    outcome: Outcome::Completion(""), // keep as valid; see next for corrupted variant
};

pub const CORRUPTED_COMMA_THEN_BRACE: Case = Case {
    name: "corrupted_comma_then_brace",
    deltas: &["{", r#""a""#, ":", "1", ",", "}"],
    outcome: Outcome::Err(Error::Corrupted),
};

// ----- Already complete snapshots -----
pub const ALREADY_COMPLETE_EMPTY_ARRAY: Case = Case {
    name: "already_complete_empty_array",
    deltas: &["[]"],
    outcome: Outcome::Completion(""),
};

pub const ALREADY_COMPLETE_SIMPLE_OBJECT: Case = Case {
    name: "already_complete_simple_object",
    deltas: &[r#"{"a":1}"#],
    outcome: Outcome::Completion(""),
};

// Collect all
pub const CASES: &[&Case] = &[
    // happy paths
    &EMPTY_ARRAY,
    &EMPTY_OBJECT,
    &ARRAY_ONE_NUMBER,
    &OBJECT_SIMPLE_KV,
    &NESTED_OBJECT_IN_ARRAY,
    &DOUBLE_NEST,
    &MULTI_KV_OBJECT_NEEDS_BRACE,
    &TRAILING_STRING_VALUE,
    &ARRAY_OF_OBJECTS_PARTIAL_SECOND,
    // not closable yet
    &OBJ_EXPECTING_COLON,
    &OBJ_EXPECTING_VALUE,
    &OBJ_IN_OPEN_STRING_VALUE,
    &OBJ_IN_ESCAPE,
    &ARRAY_AFTER_COMMA_EXPECTING_VALUE,
    &ARRAY_IN_OPEN_STRING,
    &NUMBER_PARTIAL_MINUS,
    &NUMBER_PARTIAL_EXP,
    &LITERAL_TRUE_PARTIAL,
    &LITERAL_NULL_PARTIAL,
    // corrupted
    &CORRUPTED_MISMATCH,
    &CORRUPTED_EXTRA_COLON,
    &CORRUPTED_CLOSE_BRACE_IN_ARRAY,
    &CORRUPTED_UNEXPECTED_COMMA_START_ARRAY,
    &CORRUPTED_UNEXPECTED_COMMA_START_OBJECT,
    &CORRUPTED_UNEXPECTED_COLON_TOP,
    &CORRUPTED_QUOTE_IN_NONSTRING_DATA,
    &CORRUPTED_CLOSE_BEFORE_KEY,
    &CORRUPTED_COMMA_THEN_BRACE,
    // already complete
    &ALREADY_COMPLETE_EMPTY_ARRAY,
    &ALREADY_COMPLETE_SIMPLE_OBJECT,
];
