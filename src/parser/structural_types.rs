use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum TokenProcessingError {
    NotAStructuralToken,
    NotClosable,
    NotAnOpeningOrClosingToken,
    NotAnOpeningToken,
    NotAClosingToken,
    CorruptedStackMismatchedTokens,
    CorruptedStackEmptyOnClose,
}

pub enum StructuralToken {
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    OpenKey,
    CloseKey,
    OpenStringData,
    CloseStringData,
}

impl TryFrom<&Token> for StructuralToken {
    type Error = TokenProcessingError;

    fn try_from(token: &Token) -> Result<Self, Self::Error> {
        match token {
            Token::OpenBrace => Ok(StructuralToken::OpenBrace),
            Token::CloseBrace => Ok(StructuralToken::CloseBrace),
            Token::OpenBracket => Ok(StructuralToken::OpenBracket),
            Token::CloseBracket => Ok(StructuralToken::CloseBracket),
            Token::OpenKey => Ok(StructuralToken::OpenKey),
            Token::CloseKey => Ok(StructuralToken::CloseKey),
            Token::OpenStringData => Ok(StructuralToken::OpenStringData),
            Token::CloseStringData => Ok(StructuralToken::CloseStringData),

            Token::NonStringData | Token::Comma | Token::Colon | Token::Whitespace => {
                Err(TokenProcessingError::NotAStructuralToken)
            }
        }
    }
}

#[derive(Debug)]
pub enum OpeningToken {
    OpenBrace,
    OpenBracket,
    OpenKey,
    OpenStringData,
}

#[derive(Debug, PartialEq)]
pub enum ClosingToken {
    CloseBrace,
    CloseBracket,
    CloseKey,
    CloseStringData,
}

impl OpeningToken {
    pub fn get_closing_token(&self) -> ClosingToken {
        match self {
            Self::OpenBrace => ClosingToken::CloseBrace,
            Self::OpenBracket => ClosingToken::CloseBracket,
            Self::OpenKey => ClosingToken::CloseKey,
            Self::OpenStringData => ClosingToken::CloseStringData,
        }
    }
}

impl ClosingToken {
    pub fn get_char(&self) -> char {
        match self {
            Self::CloseBrace => '}',
            Self::CloseBracket => ']',
            Self::CloseKey => '"',
            Self::CloseStringData => '"',
        }
    }
}

impl TryFrom<&StructuralToken> for OpeningToken {
    type Error = TokenProcessingError;

    fn try_from(token: &StructuralToken) -> Result<Self, Self::Error> {
        match token {
            StructuralToken::OpenBrace => Ok(OpeningToken::OpenBrace),
            StructuralToken::OpenBracket => Ok(OpeningToken::OpenBracket),
            StructuralToken::OpenKey => Ok(OpeningToken::OpenKey),
            StructuralToken::OpenStringData => Ok(OpeningToken::OpenStringData),
            _ => Err(TokenProcessingError::NotAnOpeningToken),
        }
    }
}

impl TryFrom<&StructuralToken> for ClosingToken {
    type Error = TokenProcessingError;

    fn try_from(token: &StructuralToken) -> Result<Self, Self::Error> {
        match token {
            StructuralToken::CloseBrace => Ok(ClosingToken::CloseBrace),
            StructuralToken::CloseBracket => Ok(ClosingToken::CloseBracket),
            StructuralToken::CloseKey => Ok(ClosingToken::CloseKey),
            StructuralToken::CloseStringData => Ok(ClosingToken::CloseStringData),
            _ => Err(TokenProcessingError::NotAClosingToken),
        }
    }
}
