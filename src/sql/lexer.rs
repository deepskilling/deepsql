/// SQL Lexer
/// 
/// Tokenizes SQL input into a stream of tokens

use crate::sql::tokens::{Token, TokenType};

/// SQL Lexer for tokenization
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Create a new lexer from SQL input
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token();
            let is_eof = token.token_type == TokenType::Eof;
            tokens.push(token);
            
            if is_eof {
                break;
            }
        }
        
        tokens
    }
    
    /// Get the next token
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        
        let line = self.line;
        let column = self.column;
        
        if self.is_at_end() {
            return Token::new(TokenType::Eof, line, column);
        }
        
        let ch = self.current_char();
        
        // Skip comments
        if ch == '-' && self.peek() == Some('-') {
            self.skip_line_comment();
            return self.next_token();
        }
        
        if ch == '/' && self.peek() == Some('*') {
            self.skip_block_comment();
            return self.next_token();
        }
        
        // String literals
        if ch == '\'' {
            return self.read_string(line, column);
        }
        
        // Numbers
        if ch.is_ascii_digit() {
            return self.read_number(line, column);
        }
        
        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return self.read_identifier(line, column);
        }
        
        // Operators and delimiters
        self.advance();
        
        let token_type = match ch {
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            ',' => TokenType::Comma,
            ';' => TokenType::Semicolon,
            '.' => TokenType::Dot,
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '*' => TokenType::Star,
            '/' => TokenType::Slash,
            '%' => TokenType::Percent,
            '=' => TokenType::Equal,
            '!' => {
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::NotEqual
                } else {
                    TokenType::Invalid(format!("Unexpected character: {}", ch))
                }
            }
            '<' => {
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::LessEqual
                } else if self.current_char() == '>' {
                    self.advance();
                    TokenType::NotEqual
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            _ => TokenType::Invalid(format!("Unexpected character: {}", ch)),
        };
        
        Token::new(token_type, line, column)
    }
    
    fn read_string(&mut self, line: usize, column: usize) -> Token {
        self.advance(); // Skip opening quote
        
        let mut value = String::new();
        
        while !self.is_at_end() {
            if self.current_char() == '\'' {
                // Check for SQL-style escape (two single quotes)
                if self.peek() == Some('\'') {
                    value.push('\'');
                    self.advance(); // Skip first quote
                    self.advance(); // Skip second quote
                } else {
                    // End of string
                    break;
                }
            } else if self.current_char() == '\\' {
                self.advance();
                if !self.is_at_end() {
                    value.push(self.current_char());
                    self.advance();
                }
            } else {
                value.push(self.current_char());
                self.advance();
            }
        }
        
        if self.is_at_end() {
            return Token::new(
                TokenType::Invalid("Unterminated string".to_string()),
                line,
                column,
            );
        }
        
        self.advance(); // Skip closing quote
        
        Token::new(TokenType::String(value), line, column)
    }
    
    fn read_number(&mut self, line: usize, column: usize) -> Token {
        let mut value = String::new();
        
        while !self.is_at_end() && self.current_char().is_ascii_digit() {
            value.push(self.current_char());
            self.advance();
        }
        
        // Check for decimal point
        if !self.is_at_end() && self.current_char() == '.' {
            if let Some(next) = self.peek() {
                if next.is_ascii_digit() {
                    value.push(self.current_char());
                    self.advance();
                    
                    while !self.is_at_end() && self.current_char().is_ascii_digit() {
                        value.push(self.current_char());
                        self.advance();
                    }
                }
            }
        }
        
        Token::new(TokenType::Number(value), line, column)
    }
    
    fn read_identifier(&mut self, line: usize, column: usize) -> Token {
        let mut value = String::new();
        
        while !self.is_at_end() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        // Check if it's a keyword
        let token_type = TokenType::keyword(&value)
            .unwrap_or(TokenType::Identifier(value));
        
        Token::new(token_type, line, column)
    }
    
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            let ch = self.current_char();
            if ch.is_whitespace() {
                if ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
                self.position += 1;
            } else {
                break;
            }
        }
    }
    
    fn skip_line_comment(&mut self) {
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }
    }
    
    fn skip_block_comment(&mut self) {
        self.advance(); // Skip '/'
        self.advance(); // Skip '*'
        
        while !self.is_at_end() {
            if self.current_char() == '*' && self.peek() == Some('/') {
                self.advance(); // Skip '*'
                self.advance(); // Skip '/'
                break;
            }
            self.advance();
        }
    }
    
    fn current_char(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }
    
    fn peek(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }
    
    fn advance(&mut self) {
        if !self.is_at_end() {
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_select() {
        let mut lexer = Lexer::new("SELECT * FROM users");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::Select);
        assert_eq!(tokens[1].token_type, TokenType::Star);
        assert_eq!(tokens[2].token_type, TokenType::From);
        assert_eq!(tokens[3].token_type, TokenType::Identifier("users".to_string()));
        assert_eq!(tokens[4].token_type, TokenType::Eof);
    }
    
    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("'hello world'");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::String("hello world".to_string()));
    }
    
    #[test]
    fn test_number_literal() {
        let mut lexer = Lexer::new("42 3.14");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::Number("42".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::Number("3.14".to_string()));
    }
    
    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("= != < <= > >= + - * /");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::Equal);
        assert_eq!(tokens[1].token_type, TokenType::NotEqual);
        assert_eq!(tokens[2].token_type, TokenType::Less);
        assert_eq!(tokens[3].token_type, TokenType::LessEqual);
        assert_eq!(tokens[4].token_type, TokenType::Greater);
        assert_eq!(tokens[5].token_type, TokenType::GreaterEqual);
    }
    
    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("SELECT -- this is a comment\n* FROM users");
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0].token_type, TokenType::Select);
        assert_eq!(tokens[1].token_type, TokenType::Star);
        assert_eq!(tokens[2].token_type, TokenType::From);
    }
}

