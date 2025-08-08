use state_types::{ClosingToken, JSONState};

mod get_balancing_chars;
mod handle_escape;
mod is_valid_non_string_data;
mod modify_stack;
mod non_string_data;
mod parse_brace;
mod parse_bracket;
mod parse_char;
mod parse_colon;
mod parse_comma;
mod parse_error_types;
mod parse_quote_char;
mod state_types;
mod string_data;
mod structure_type;

//pub struct JSONBalancer<'a> {
//    closing_stack: &'a [ClosingToken],
//    state: &'a JSONState,
//}
//
//impl<'a> JSONBalancer<'a> {
//    pub fn new() -> Self {
//        JSONBalancer {}
//    }
//}

fn main() {}
