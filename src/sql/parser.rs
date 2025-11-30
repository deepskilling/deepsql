/// SQL Parser
/// 
/// Parses tokens into AST

use crate::error::{Error, Result};
use crate::sql::ast::*;
use crate::sql::tokens::{Token, TokenType};

/// SQL Parser - converts tokens to AST
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Create a new parser from tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }
    
    /// Parse a SQL statement
    pub fn parse_statement(&mut self) -> Result<Statement> {
        let token = self.current();
        
        match &token.token_type {
            TokenType::Select => Ok(Statement::Select(self.parse_select()?)),
            TokenType::Insert => Ok(Statement::Insert(self.parse_insert()?)),
            TokenType::Update => Ok(Statement::Update(self.parse_update()?)),
            TokenType::Delete => Ok(Statement::Delete(self.parse_delete()?)),
            TokenType::Create => Ok(Statement::CreateTable(self.parse_create_table()?)),
            _ => Err(Error::Internal(format!("Unexpected token: {:?}", token.token_type))),
        }
    }
    
    /// Parse SELECT statement
    fn parse_select(&mut self) -> Result<SelectStatement> {
        self.expect(TokenType::Select)?;
        
        let mut stmt = SelectStatement::new();
        
        // Check for DISTINCT
        if self.check(&TokenType::Distinct) {
            self.advance();
            stmt.distinct = true;
        }
        
        // Parse columns
        if self.check(&TokenType::Star) {
            self.advance();
            stmt.columns.push(SelectColumn::Star);
        } else {
            loop {
                let expr = self.parse_expression()?;
                let alias = if self.check(&TokenType::As) {
                    self.advance();
                    Some(self.expect_identifier()?)
                } else {
                    None
                };
                
                stmt.columns.push(SelectColumn::Expr { expr, alias });
                
                if !self.check(&TokenType::Comma) {
                    break;
                }
                self.advance();
            }
        }
        
        // Parse FROM clause
        if self.check(&TokenType::From) {
            self.advance();
            stmt.from = Some(self.expect_identifier()?);
        }
        
        // Parse WHERE clause
        if self.check(&TokenType::Where) {
            self.advance();
            stmt.where_clause = Some(self.parse_expression()?);
        }
        
        // Parse ORDER BY clause
        if self.check(&TokenType::Order) {
            self.advance();
            if self.check(&TokenType::By) {
                self.advance();
            }
            
            loop {
                let expr = self.parse_expression()?;
                let direction = if self.current_matches_identifier("DESC") {
                    self.advance();
                    OrderDirection::Desc
                } else if self.current_matches_identifier("ASC") {
                    self.advance();
                    OrderDirection::Asc
                } else {
                    OrderDirection::Asc
                };
                
                stmt.order_by.push(OrderBy { expr, direction });
                
                if !self.check(&TokenType::Comma) {
                    break;
                }
                self.advance();
            }
        }
        
        // Parse LIMIT clause
        if self.check(&TokenType::Limit) {
            self.advance();
            stmt.limit = Some(self.expect_number()? as usize);
        }
        
        // Parse OFFSET clause
        if self.check(&TokenType::Offset) {
            self.advance();
            stmt.offset = Some(self.expect_number()? as usize);
        }
        
        Ok(stmt)
    }
    
    /// Parse INSERT statement
    fn parse_insert(&mut self) -> Result<InsertStatement> {
        self.expect(TokenType::Insert)?;
        self.expect(TokenType::Into)?;
        
        let table = self.expect_identifier()?;
        let mut stmt = InsertStatement::new(table);
        
        // Parse optional column list
        if self.check(&TokenType::LeftParen) {
            self.advance();
            let mut columns = Vec::new();
            
            loop {
                columns.push(self.expect_identifier()?);
                
                if !self.check(&TokenType::Comma) {
                    break;
                }
                self.advance();
            }
            
            self.expect(TokenType::RightParen)?;
            stmt.columns = Some(columns);
        }
        
        // Parse VALUES
        self.expect(TokenType::Values)?;
        
        loop {
            self.expect(TokenType::LeftParen)?;
            
            let mut values = Vec::new();
            loop {
                values.push(self.parse_expression()?);
                
                if !self.check(&TokenType::Comma) {
                    break;
                }
                self.advance();
            }
            
            self.expect(TokenType::RightParen)?;
            stmt.values.push(values);
            
            if !self.check(&TokenType::Comma) {
                break;
            }
            self.advance();
        }
        
        Ok(stmt)
    }
    
    /// Parse UPDATE statement
    fn parse_update(&mut self) -> Result<UpdateStatement> {
        self.expect(TokenType::Update)?;
        
        let table = self.expect_identifier()?;
        let mut stmt = UpdateStatement::new(table);
        
        self.expect(TokenType::Set)?;
        
        // Parse SET column = value pairs
        loop {
            let column = self.expect_identifier()?;
            self.expect(TokenType::Equal)?;
            let value = self.parse_expression()?;
            
            stmt.assignments.push(Assignment { column, value });
            
            if !self.check(&TokenType::Comma) {
                break;
            }
            self.advance();
        }
        
        // Parse WHERE clause
        if self.check(&TokenType::Where) {
            self.advance();
            stmt.where_clause = Some(self.parse_expression()?);
        }
        
        Ok(stmt)
    }
    
    /// Parse DELETE statement
    fn parse_delete(&mut self) -> Result<DeleteStatement> {
        self.expect(TokenType::Delete)?;
        self.expect(TokenType::From)?;
        
        let table = self.expect_identifier()?;
        let mut stmt = DeleteStatement::new(table);
        
        // Parse WHERE clause
        if self.check(&TokenType::Where) {
            self.advance();
            stmt.where_clause = Some(self.parse_expression()?);
        }
        
        Ok(stmt)
    }
    
    /// Parse CREATE TABLE statement
    fn parse_create_table(&mut self) -> Result<CreateTableStatement> {
        self.expect(TokenType::Create)?;
        self.expect(TokenType::Table)?;
        
        let table = self.expect_identifier()?;
        let mut stmt = CreateTableStatement::new(table);
        
        self.expect(TokenType::LeftParen)?;
        
        // Parse column definitions
        loop {
            let column_name = self.expect_identifier()?;
            let data_type = self.parse_data_type()?;
            
            let mut column_def = ColumnDef::new(column_name, data_type);
            
            // Parse constraints
            while !self.check(&TokenType::Comma) && !self.check(&TokenType::RightParen) {
                // Handle NOT NULL (NOT is a keyword token)
                if self.check(&TokenType::Not) {
                    self.advance();
                    if self.check(&TokenType::Null) {
                        self.advance();
                        column_def.constraints.push(ColumnConstraint::NotNull);
                    }
                } else if self.current_matches_identifier("PRIMARY") {
                    self.advance();
                    if self.current_matches_identifier("KEY") {
                        self.advance();
                        column_def.constraints.push(ColumnConstraint::PrimaryKey);
                    }
                } else if self.current_matches_identifier("UNIQUE") {
                    self.advance();
                    column_def.constraints.push(ColumnConstraint::Unique);
                } else {
                    break;
                }
            }
            
            stmt.columns.push(column_def);
            
            if !self.check(&TokenType::Comma) {
                break;
            }
            self.advance();
        }
        
        self.expect(TokenType::RightParen)?;
        
        Ok(stmt)
    }
    
    /// Parse data type
    fn parse_data_type(&mut self) -> Result<DataType> {
        let token = self.current();
        
        let data_type = match &token.token_type {
            TokenType::Integer => DataType::Integer,
            TokenType::Real => DataType::Real,
            TokenType::Text => DataType::Text,
            TokenType::Blob => DataType::Blob,
            TokenType::Identifier(name) => {
                match name.to_uppercase().as_str() {
                    "INT" => DataType::Integer,
                    "FLOAT" | "DOUBLE" => DataType::Real,
                    "VARCHAR" | "CHAR" => DataType::Text,
                    _ => return Err(Error::Internal(format!("Unknown data type: {}", name))),
                }
            }
            _ => return Err(Error::Internal(format!("Expected data type, got: {:?}", token.token_type))),
        };
        
        self.advance();
        Ok(data_type)
    }
    
    /// Parse expression with operator precedence
    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_or()
    }
    
    fn parse_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_and()?;
        
        while self.check(&TokenType::Or) {
            self.advance();
            let right = self.parse_and()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: BinaryOperator::Or,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_equality()?;
        
        while self.check(&TokenType::And) {
            self.advance();
            let right = self.parse_equality()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: BinaryOperator::And,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr = self.parse_comparison()?;
        
        while matches!(self.current().token_type, TokenType::Equal | TokenType::NotEqual) {
            let op = match self.current().token_type {
                TokenType::Equal => BinaryOperator::Equal,
                TokenType::NotEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_comparison()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.parse_addition()?;
        
        while matches!(
            self.current().token_type,
            TokenType::Less | TokenType::LessEqual | TokenType::Greater | TokenType::GreaterEqual
        ) {
            let op = match self.current().token_type {
                TokenType::Less => BinaryOperator::Less,
                TokenType::LessEqual => BinaryOperator::LessEqual,
                TokenType::Greater => BinaryOperator::Greater,
                TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_addition()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_addition(&mut self) -> Result<Expr> {
        let mut expr = self.parse_multiplication()?;
        
        while matches!(self.current().token_type, TokenType::Plus | TokenType::Minus) {
            let op = match self.current().token_type {
                TokenType::Plus => BinaryOperator::Add,
                TokenType::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_multiplication()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_multiplication(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;
        
        while matches!(self.current().token_type, TokenType::Star | TokenType::Slash | TokenType::Percent) {
            let op = match self.current().token_type {
                TokenType::Star => BinaryOperator::Multiply,
                TokenType::Slash => BinaryOperator::Divide,
                TokenType::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_unary(&mut self) -> Result<Expr> {
        if matches!(self.current().token_type, TokenType::Not | TokenType::Minus) {
            let op = match self.current().token_type {
                TokenType::Not => UnaryOperator::Not,
                TokenType::Minus => UnaryOperator::Minus,
                _ => unreachable!(),
            };
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryOp {
                op,
                expr: Box::new(expr),
            });
        }
        
        self.parse_primary()
    }
    
    fn parse_primary(&mut self) -> Result<Expr> {
        let token_type = self.current().token_type.clone();
        
        match token_type {
            TokenType::Number(n) => {
                self.advance();
                if n.contains('.') {
                    Ok(Expr::Literal(Literal::Real(n.parse().unwrap())))
                } else {
                    Ok(Expr::Literal(Literal::Integer(n.parse().unwrap())))
                }
            }
            TokenType::String(s) => {
                self.advance();
                Ok(Expr::Literal(Literal::String(s)))
            }
            TokenType::Null => {
                self.advance();
                Ok(Expr::Literal(Literal::Null))
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(true)))
            }
            TokenType::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(false)))
            }
            TokenType::Identifier(name) => {
                let name = name;
                self.advance();
                
                // Check for table.column
                if self.check(&TokenType::Dot) {
                    self.advance();
                    let column = self.expect_identifier()?;
                    Ok(Expr::Column {
                        table: Some(name),
                        name: column,
                    })
                }
                // Check for function call
                else if self.check(&TokenType::LeftParen) {
                    self.advance();
                    let mut args = Vec::new();
                    
                    if !self.check(&TokenType::RightParen) {
                        // Special handling for COUNT(*)
                        if self.check(&TokenType::Star) && name.to_uppercase() == "COUNT" {
                            self.advance();
                            // Use a special marker for COUNT(*) - empty args vector
                            // We'll handle this specially in the compiler
                        } else {
                            loop {
                                args.push(self.parse_expression()?);
                                if !self.check(&TokenType::Comma) {
                                    break;
                                }
                                self.advance();
                            }
                        }
                    }
                    
                    self.expect(TokenType::RightParen)?;
                    Ok(Expr::Function { name, args })
                } else {
                    Ok(Expr::Column {
                        table: None,
                        name,
                    })
                }
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenType::RightParen)?;
                Ok(expr)
            }
            _ => Err(Error::Internal(format!("Unexpected token in expression: {:?}", token_type))),
        }
    }
    
    // Helper methods
    
    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }
    
    #[allow(dead_code)]
    fn previous(&self) -> &Token {
        &self.tokens[self.position.saturating_sub(1)]
    }
    
    fn check(&self, token_type: &TokenType) -> bool {
        std::mem::discriminant(&self.current().token_type) == std::mem::discriminant(token_type)
    }
    
    fn advance(&mut self) {
        if self.position < self.tokens.len() - 1 {
            self.position += 1;
        }
    }
    
    fn expect(&mut self, token_type: TokenType) -> Result<()> {
        if self.check(&token_type) {
            self.advance();
            Ok(())
        } else {
            Err(Error::Internal(format!(
                "Expected {:?}, got {:?}",
                token_type,
                self.current().token_type
            )))
        }
    }
    
    fn expect_identifier(&mut self) -> Result<String> {
        match &self.current().token_type {
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(Error::Internal(format!(
                "Expected identifier, got {:?}",
                self.current().token_type
            ))),
        }
    }
    
    fn expect_number(&mut self) -> Result<i64> {
        match &self.current().token_type {
            TokenType::Number(n) => {
                let num = n.parse().map_err(|_| Error::Internal("Invalid number".to_string()))?;
                self.advance();
                Ok(num)
            }
            _ => Err(Error::Internal(format!(
                "Expected number, got {:?}",
                self.current().token_type
            ))),
        }
    }
    
    fn current_matches_identifier(&self, name: &str) -> bool {
        match &self.current().token_type {
            TokenType::Identifier(id) => id.to_uppercase() == name.to_uppercase(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::lexer::Lexer;
    
    #[test]
    fn test_parse_simple_select() {
        let mut lexer = Lexer::new("SELECT * FROM users");
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        
        let stmt = parser.parse_statement().unwrap();
        
        match stmt {
            Statement::Select(select) => {
                assert_eq!(select.columns.len(), 1);
                assert_eq!(select.from, Some("users".to_string()));
            }
            _ => panic!("Expected SELECT statement"),
        }
    }
    
    #[test]
    fn test_parse_insert() {
        let mut lexer = Lexer::new("INSERT INTO users (name, age) VALUES ('Alice', 30)");
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        
        let stmt = parser.parse_statement().unwrap();
        
        match stmt {
            Statement::Insert(insert) => {
                assert_eq!(insert.table, "users");
                assert_eq!(insert.columns, Some(vec!["name".to_string(), "age".to_string()]));
                assert_eq!(insert.values.len(), 1);
            }
            _ => panic!("Expected INSERT statement"),
        }
    }
    
    #[test]
    fn test_parse_create_table() {
        let mut lexer = Lexer::new("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)");
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        
        let stmt = parser.parse_statement().unwrap();
        
        match stmt {
            Statement::CreateTable(create) => {
                assert_eq!(create.table, "users");
                assert_eq!(create.columns.len(), 2);
            }
            _ => panic!("Expected CREATE TABLE statement"),
        }
    }
}

