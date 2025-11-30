/// Expression tree for WHERE clauses, ORDER BY, etc.

#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum Expr {
    /// Literal value
    Literal(Literal),
    
    /// Column reference (table.column or just column)
    Column {
        table: Option<String>,
        name: String,
    },
    
    /// Binary operation (a + b, a = b, etc.)
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
    
    /// Unary operation (NOT x, -x)
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expr>,
    },
    
    /// Function call
    Function {
        name: String,
        args: Vec<Expr>,
    },
}

/// Literal value types
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum Literal {
    /// Integer literal
    Integer(i64),
    
    /// Float literal
    Real(f64),
    
    /// String literal
    String(String),
    
    /// NULL
    Null,
    
    /// Boolean
    Boolean(bool),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum BinaryOperator {
    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    
    // Logical
    And,
    Or,
    
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum UnaryOperator {
    Not,
    Minus,
}

/// ORDER BY direction
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum OrderDirection {
    Asc,
    Desc,
}

/// ORDER BY clause
#[derive(Debug, Clone, PartialEq)]
pub struct OrderBy {
    /// Expression to order by
    pub expr: Expr,
    /// Sort direction (ASC or DESC)
    pub direction: OrderDirection,
}

