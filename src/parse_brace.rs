use crate::parse_error_types::JSONParseError;
use crate::state_types::*;
use crate::structure_type::RecursiveStructureType;

pub fn parse_brace(
    brace: RecursiveStructureType,
    current_state: &mut JSONState,
) -> Result<Token, JSONParseError> {
    todo!()
}
