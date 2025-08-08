pub enum RecursiveStructureType {
    Open,
    Close,
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
