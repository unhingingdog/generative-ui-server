#[derive(Debug, PartialEq, Clone)]
pub enum StringState {
    Open,
    Closed,
    Escaped,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrimValue {
    String(StringState),
    NonString,
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
    InValue(PrimValue),
    ExpectingValue,
}

#[derive(Debug, PartialEq, Clone)]
pub enum JSONState {
    Brace(BraceState),
    Bracket(BracketState),
    Pending,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    OpenBrace,    // '{' : puts into BraceState
    CloseBrace,   // '}' : exits braceState or pops off stack if hit as first current state
    OpenBracket,  // '[' :
    CloseBracket, // ']'
    OpenKey,      // '"' if not already open
    CloseKey,     // '"' if already open
    OpenStringData,
    CloseStringData,
    NonStringData, // on hitting first char of a number or null in a value
    Comma,         // ','
    Colon,         // ':'
    Whitespace,
}
