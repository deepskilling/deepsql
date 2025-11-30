/// SQL Token types
/// 
/// Defines all token types recognized by the SQL lexer

#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum TokenType {
    // Keywords
    Select,
    From,
    Where,
    Insert,
    Into,
    Values,
    Update,
    Set,
    Delete,
    Create,
    Table,
    Order,
    By,
    GroupBy,
    Group,
    Having,
    Limit,
    Offset,
    And,
    Or,
    Not,
    As,
    Distinct,
    Join,
    Inner,
    Left,
    Right,
    On,
    
    // Data types
    Integer,
    Real,
    Text,
    Blob,
    
    // Literals
    Number(String),
    String(String),
    Null,
    True,
    False,
    
    // Identifiers
    Identifier(String),
    
    // Operators
    Equal,           // =
    NotEqual,        // != or <>
    Less,            // <
    LessEqual,       // <=
    Greater,         // >
    GreaterEqual,    // >=
    Plus,            // +
    Minus,           // -
    Star,            // *
    Slash,           // /
    Percent,         // %
    
    // Delimiters
    LeftParen,       // (
    RightParen,      // )
    Comma,           // ,
    Semicolon,       // ;
    Dot,             // .
    
    // Special
    Eof,
    Invalid(String),
}

/// Represents a lexed token with position information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The type of token
    pub token_type: TokenType,
    /// Line number in source
    pub line: usize,
    /// Column number in source
    pub column: usize,
}

impl Token {
    /// Create a new token
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Token {
            token_type,
            line,
            column,
        }
    }
}

impl TokenType {
    /// Check if this is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenType::Select
                | TokenType::From
                | TokenType::Where
                | TokenType::Insert
                | TokenType::Into
                | TokenType::Values
                | TokenType::Update
                | TokenType::Set
                | TokenType::Delete
                | TokenType::Create
                | TokenType::Table
                | TokenType::Order
                | TokenType::By
                | TokenType::Group
                | TokenType::Having
                | TokenType::Limit
                | TokenType::Offset
                | TokenType::And
                | TokenType::Or
                | TokenType::Not
        )
    }
    
    /// Get keyword from string
    pub fn keyword(word: &str) -> Option<TokenType> {
        match word.to_uppercase().as_str() {
            "SELECT" => Some(TokenType::Select),
            "FROM" => Some(TokenType::From),
            "WHERE" => Some(TokenType::Where),
            "INSERT" => Some(TokenType::Insert),
            "INTO" => Some(TokenType::Into),
            "VALUES" => Some(TokenType::Values),
            "UPDATE" => Some(TokenType::Update),
            "SET" => Some(TokenType::Set),
            "DELETE" => Some(TokenType::Delete),
            "CREATE" => Some(TokenType::Create),
            "TABLE" => Some(TokenType::Table),
            "ORDER" => Some(TokenType::Order),
            "BY" => Some(TokenType::By),
            "GROUP" => Some(TokenType::Group),
            "HAVING" => Some(TokenType::Having),
            "LIMIT" => Some(TokenType::Limit),
            "OFFSET" => Some(TokenType::Offset),
            "AND" => Some(TokenType::And),
            "OR" => Some(TokenType::Or),
            "NOT" => Some(TokenType::Not),
            "AS" => Some(TokenType::As),
            "DISTINCT" => Some(TokenType::Distinct),
            "JOIN" => Some(TokenType::Join),
            "INNER" => Some(TokenType::Inner),
            "LEFT" => Some(TokenType::Left),
            "RIGHT" => Some(TokenType::Right),
            "ON" => Some(TokenType::On),
            "INTEGER" => Some(TokenType::Integer),
            "INT" => Some(TokenType::Integer),
            "REAL" => Some(TokenType::Real),
            "FLOAT" => Some(TokenType::Real),
            "TEXT" => Some(TokenType::Text),
            "VARCHAR" => Some(TokenType::Text),
            "BLOB" => Some(TokenType::Blob),
            "NULL" => Some(TokenType::Null),
            "TRUE" => Some(TokenType::True),
            "FALSE" => Some(TokenType::False),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keyword_recognition() {
        assert_eq!(TokenType::keyword("SELECT"), Some(TokenType::Select));
        assert_eq!(TokenType::keyword("select"), Some(TokenType::Select));
        assert_eq!(TokenType::keyword("FROM"), Some(TokenType::From));
        assert_eq!(TokenType::keyword("unknown"), None);
    }
    
    #[test]
    fn test_is_keyword() {
        assert!(TokenType::Select.is_keyword());
        assert!(TokenType::Where.is_keyword());
        assert!(!TokenType::Identifier("test".to_string()).is_keyword());
    }
}

