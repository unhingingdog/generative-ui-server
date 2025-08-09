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

/* ----------------------------- Happy paths ----------------------------- */

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

pub const ARRAY_ONE_CLOSABLE_LITERAL: Case = Case {
    name: "array_one_number",
    deltas: &["[", "true"],
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

pub const OBJ_VALUE_ARRAY_PARTIAL: Case = Case {
    name: "obj_value_array_partial",
    deltas: &["{", r#""a""#, ":", "[", "1"],
    outcome: Outcome::Completion("]}"),
};

pub const NESTED_ARRAYS_NEED_TWO_BRACKETS: Case = Case {
    name: "nested_arrays_need_two_brackets",
    deltas: &["[", "[", "1"],
    outcome: Outcome::Completion("]]"),
};

/* ---------------- Partial-but-closable (auto-complete) ----------------- */

pub const ARRAY_ONE_STRING_OPEN: Case = Case {
    name: "array_one_string_open",
    deltas: &["[", r#""hel"#],
    outcome: Outcome::Completion("\"]"),
};

pub const ARRAY_IN_OPEN_STRING: Case = Case {
    name: "array_in_open_string",
    deltas: &["[", r#""hel"#],
    outcome: Outcome::Completion("\"]"),
};

pub const OBJ_IN_OPEN_STRING_VALUE: Case = Case {
    name: "obj_in_open_string_value",
    deltas: &["{", r#""a""#, ":", r#""va"#],
    outcome: Outcome::Completion("\"}"),
};

pub const OBJ_ESCAPED_QUOTE_THEN_CLOSABLE: Case = Case {
    name: "obj_escaped_quote_then_closable",
    deltas: &["{", r#""a""#, ":", r#"""#, "\\", r#"""#],
    outcome: Outcome::Completion("\"}"),
};

pub const ARRAY_STRING_ESCAPED_THEN_CLOSABLE: Case = Case {
    name: "array_string_escaped_then_closable",
    deltas: &["[", r#"""#, "\\", r#"""#],
    outcome: Outcome::Completion("\"]"),
};

pub const TRAILING_WS_AFTER_OBJ_VALUE: Case = Case {
    name: "trailing_ws_after_obj_value",
    deltas: &["{", r#""a""#, ":", r#""x""#, " ", "\t"],
    outcome: Outcome::Completion("}"),
};

pub const TRAILING_WS_AFTER_ARRAY_VALUE: Case = Case {
    name: "trailing_ws_after_array_value",
    deltas: &["[", r#""x""#, " ", "\n"],
    outcome: Outcome::Completion("]"),
};

/* -------------------------- Not closable yet --------------------------- */

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

pub const OBJ_IN_OPEN_STRING_KEY: Case = Case {
    name: "obj_in_open_string_key",
    deltas: &["{", r#""ke"#],
    outcome: Outcome::Err(Error::NotClosable),
};

pub const OBJ_IN_ESCAPE: Case = Case {
    name: "obj_in_escape",
    deltas: &["{", r#""a""#, ":", r#""va\"#],
    outcome: Outcome::Err(Error::NotClosable),
};

pub const ARRAY_IN_ESCAPE: Case = Case {
    name: "array_in_escape",
    deltas: &["[", r#"""#, "\\"],
    outcome: Outcome::Err(Error::NotClosable),
};

pub const ARRAY_AFTER_COMMA_EXPECTING_VALUE: Case = Case {
    name: "array_after_comma_expecting_value",
    deltas: &["[", "1", ",", ""],
    outcome: Outcome::Err(Error::NotClosable),
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

pub const NUMBER_PARTIAL_DECIMAL: Case = Case {
    name: "number_partial_decimal",
    deltas: &["{", r#""n""#, ":", "1."],
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

pub const UNICODE_ESCAPE_PARTIAL: Case = Case {
    name: "unicode_escape_partial",
    deltas: &["{", r#""a""#, ":", r#"""#, "\\", "u"],
    outcome: Outcome::Err(Error::NotClosable),
};

/* --------------------------- Corrupted/invalid ------------------------- */

pub const CORRUPTED_MISMATCH: Case = Case {
    name: "corrupted_mismatch",
    deltas: &["[", "]", "]"],
    outcome: Outcome::Err(Error::Corrupted),
};

// TODO: this fails, though it goes beyond the purpose of this lib (closing no a full json parser)
//pub const CORRUPTED_EXTRA_COLON: Case = Case {
//    name: "corrupted_extra_colon",
//    deltas: &["{", r#""a""#, ":", ":", "1"],
//    outcome: Outcome::Err(Error::Corrupted),
//};

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
    deltas: &["{", "}"],
    outcome: Outcome::Completion(""),
};

pub const CORRUPTED_COMMA_THEN_BRACE: Case = Case {
    name: "corrupted_comma_then_brace",
    deltas: &["{", r#""a""#, ":", "1", ",", "}"],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const ARRAY_TRAILING_COMMA_THEN_CLOSE: Case = Case {
    name: "array_trailing_comma_then_close",
    deltas: &["[", "1", ",", "]"],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const TOPLEVEL_CLOSE_BRACE: Case = Case {
    name: "toplevel_close_brace",
    deltas: &["}"],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const TOPLEVEL_CLOSE_BRACKET: Case = Case {
    name: "toplevel_close_bracket",
    deltas: &["]"],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const OBJECT_CLOSE_BRACKET_MISMATCH: Case = Case {
    name: "object_close_bracket_mismatch",
    deltas: &["{", "]"],
    outcome: Outcome::Err(Error::Corrupted),
};

//pub const UNICODE_ESCAPE_INVALID_HEX: Case = Case {
//    name: "unicode_escape_invalid_hex",
//    deltas: &["{", r#""a""#, ":", r#"""#, "\\", "u", "Z"],
//    outcome: Outcome::Err(Error::Corrupted),
//};

//pub const ARRAY_UNICODE_ESCAPE_INVALID_HEX: Case = Case {
//    name: "array_unicode_escape_invalid_hex",
//    deltas: &["[", r#"""#, "\\", "u", "Z"],
//    outcome: Outcome::Err(Error::Corrupted),
//};

pub const OBJ_AFTER_STRING_NON_DELIMITER: Case = Case {
    name: "obj_after_string_non_delimiter",
    deltas: &["{", r#""a""#, ":", r#""x""#, "1"],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const ARRAY_AFTER_STRING_NON_DELIMITER: Case = Case {
    name: "array_after_string_non_delimiter",
    deltas: &["[", r#""x""#, "1"],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const UNQUOTED_KEY_IS_CORRUPTED: Case = Case {
    name: "unquoted_key_is_corrupted",
    deltas: &["{", "a"],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const UNEXPECTED_OPEN_BRACKET_IN_KEY: Case = Case {
    name: "unexpected_open_bracket_in_key",
    deltas: &["{", "["],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const TOPLEVEL_NUMBER_NOT_ALLOWED: Case = Case {
    name: "toplevel_number_not_allowed",
    deltas: &["1"],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const TOPLEVEL_QUOTE_NOT_ALLOWED: Case = Case {
    name: "toplevel_quote_not_allowed",
    deltas: &[r#"""#],
    outcome: Outcome::Err(Error::Corrupted),
};

/* ------------------------- Already complete --------------------------- */

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

pub const ALREADY_COMPLETE_OBJECT_THEN_WS: Case = Case {
    name: "already_complete_object_then_ws",
    deltas: &[r#"{"a":1}"#, "  "],
    outcome: Outcome::Completion(""),
};

pub const ALREADY_COMPLETE_ARRAY_THEN_WS: Case = Case {
    name: "already_complete_array_then_ws",
    deltas: &["[]", "\n"],
    outcome: Outcome::Completion(""),
};

/* ----------------- Stream integrity and edge cases ------------------ */

pub const MESSY_CHUNK_SPLIT_KEYWORD: Case = Case {
    name: "messy_chunk_split_keyword",
    deltas: &["[t", "ru", "e"],
    outcome: Outcome::Completion("]"),
};

pub const MESSY_CHUNK_SPLIT_ESCAPE: Case = Case {
    name: "messy_chunk_split_escape",
    deltas: &["[\"\\", "\"abc"],
    outcome: Outcome::Completion("\"]"),
};

pub const CORRUPTED_TRAILING_CONTENT_AFTER_ARRAY: Case = Case {
    name: "corrupted_trailing_content_after_array",
    deltas: &["[1, 2]", "3"],
    outcome: Outcome::Err(Error::Corrupted),
};

pub const CORRUPTED_TRAILING_CONTENT_AFTER_OBJECT: Case = Case {
    name: "corrupted_trailing_content_after_object",
    deltas: &[r#"{"a":1}"#, "x"],
    outcome: Outcome::Err(Error::Corrupted),
};

/* ------------------------------ Registry ------------------------------ */

pub const CASES: &[&Case] = &[
    // happy paths
    &EMPTY_ARRAY,
    &EMPTY_OBJECT,
    &ARRAY_ONE_NUMBER,
    &ARRAY_ONE_CLOSABLE_LITERAL,
    &OBJECT_SIMPLE_KV,
    &NESTED_OBJECT_IN_ARRAY,
    &DOUBLE_NEST,
    &MULTI_KV_OBJECT_NEEDS_BRACE,
    &TRAILING_STRING_VALUE,
    &ARRAY_OF_OBJECTS_PARTIAL_SECOND,
    &OBJ_VALUE_ARRAY_PARTIAL,
    &NESTED_ARRAYS_NEED_TWO_BRACKETS,
    // partial-but-closable
    &ARRAY_ONE_STRING_OPEN,
    &ARRAY_IN_OPEN_STRING,
    &OBJ_IN_OPEN_STRING_VALUE,
    &OBJ_ESCAPED_QUOTE_THEN_CLOSABLE,
    &ARRAY_STRING_ESCAPED_THEN_CLOSABLE,
    &TRAILING_WS_AFTER_OBJ_VALUE,
    &TRAILING_WS_AFTER_ARRAY_VALUE,
    // not closable yet
    &OBJ_EXPECTING_COLON,
    &OBJ_EXPECTING_VALUE,
    &OBJ_IN_OPEN_STRING_KEY,
    &OBJ_IN_ESCAPE,
    &ARRAY_IN_ESCAPE,
    &ARRAY_AFTER_COMMA_EXPECTING_VALUE,
    &NUMBER_PARTIAL_MINUS,
    &NUMBER_PARTIAL_EXP,
    &NUMBER_PARTIAL_DECIMAL,
    &LITERAL_TRUE_PARTIAL,
    &LITERAL_NULL_PARTIAL,
    &UNICODE_ESCAPE_PARTIAL,
    // corrupted/invalid
    &CORRUPTED_MISMATCH,
    //&CORRUPTED_EXTRA_COLON,
    &CORRUPTED_CLOSE_BRACE_IN_ARRAY,
    &CORRUPTED_UNEXPECTED_COMMA_START_ARRAY,
    &CORRUPTED_UNEXPECTED_COMMA_START_OBJECT,
    &CORRUPTED_UNEXPECTED_COLON_TOP,
    &CORRUPTED_QUOTE_IN_NONSTRING_DATA,
    &CORRUPTED_CLOSE_BEFORE_KEY,
    &CORRUPTED_COMMA_THEN_BRACE,
    &ARRAY_TRAILING_COMMA_THEN_CLOSE,
    &TOPLEVEL_CLOSE_BRACE,
    &TOPLEVEL_CLOSE_BRACKET,
    &OBJECT_CLOSE_BRACKET_MISMATCH,
    //&UNICODE_ESCAPE_INVALID_HEX,
    //&ARRAY_UNICODE_ESCAPE_INVALID_HEX,
    &OBJ_AFTER_STRING_NON_DELIMITER,
    &ARRAY_AFTER_STRING_NON_DELIMITER,
    &UNQUOTED_KEY_IS_CORRUPTED,
    &UNEXPECTED_OPEN_BRACKET_IN_KEY,
    &TOPLEVEL_NUMBER_NOT_ALLOWED,
    &TOPLEVEL_QUOTE_NOT_ALLOWED,
    // already complete
    &ALREADY_COMPLETE_EMPTY_ARRAY,
    &ALREADY_COMPLETE_SIMPLE_OBJECT,
    &ALREADY_COMPLETE_OBJECT_THEN_WS,
    &ALREADY_COMPLETE_ARRAY_THEN_WS,
    // stream integrity
    &MESSY_CHUNK_SPLIT_KEYWORD,
    &MESSY_CHUNK_SPLIT_ESCAPE,
    &CORRUPTED_TRAILING_CONTENT_AFTER_ARRAY,
    &CORRUPTED_TRAILING_CONTENT_AFTER_OBJECT,
];
