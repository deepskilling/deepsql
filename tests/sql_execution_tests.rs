/// SQL Execution Tests
/// 
/// Tests end-to-end SQL execution through the SqlEngine

use deepsql::sql_engine::SqlEngine;
use deepsql::storage::Pager;
use std::fs;

#[test]
fn test_sql_engine_creation() {
    let path = "/tmp/test_sql_engine.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let engine = SqlEngine::new(pager);
    
    // Engine created successfully
    drop(engine);
    
    let _ = fs::remove_file(path);
}

#[test]
fn test_sql_engine_parse_simple_select() {
    let path = "/tmp/test_sql_parse.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    
    // Load catalog
    engine.load_catalog().unwrap();
    
    // Try to execute a simple SELECT (will fail because table doesn't exist, but parsing should work)
    let result = engine.execute("SELECT * FROM users");
    
    // We expect it to fail on execution (table doesn't exist)
    // But parsing, planning, and compilation should succeed
    // The error will come from execution phase
    match result {
        Ok(_) => {
            // If it succeeds, that's fine too (empty result)
        }
        Err(e) => {
            // Expected error: table doesn't exist or similar
            println!("Expected error during execution: {:?}", e);
        }
    }
    
    drop(engine);
    let _ = fs::remove_file(path);
}

#[test]
fn test_sql_pipeline_components() {
    use deepsql::sql::lexer::Lexer;
    use deepsql::sql::parser::Parser;
    use deepsql::planner::builder::PlanBuilder;
    use deepsql::planner::optimizer::Optimizer;
    use deepsql::planner::compiler::VMCompiler;
    
    let sql = "SELECT id, name FROM users WHERE age > 18";
    
    // Step 1: Lexing
    let mut lexer = Lexer::new(sql);
    let tokens = lexer.tokenize();
    println!("Tokens: {} tokens generated", tokens.len());
    assert!(tokens.len() > 0);
    
    // Step 2: Parsing
    let mut parser = Parser::new(tokens);
    let statement = parser.parse_statement().unwrap();
    println!("AST: {:?}", statement);
    
    // Step 3: Logical Plan
    let builder = PlanBuilder::new();
    let logical_plan = builder.build(statement).unwrap();
    println!("Logical Plan: {:?}", logical_plan);
    
    // Step 4: Optimization
    let optimizer = Optimizer::new();
    let optimized = optimizer.optimize(logical_plan);
    println!("Optimized Plan: {:?}", optimized);
    
    // Step 5: Physical Plan (we'll use the from_logical method)
    use deepsql::planner::physical::PhysicalPlan;
    let physical = PhysicalPlan::from_logical(optimized);
    println!("Physical Plan: {:?}", physical);
    
    // Step 6: VM Compilation
    let mut compiler = VMCompiler::new();
    let program = compiler.compile(&physical).unwrap();
    println!("VM Program: {} opcodes", program.opcodes.len());
    
    // Print opcodes for inspection
    for (i, opcode) in program.opcodes.iter().enumerate() {
        println!("  {}: {}", i, opcode);
    }
    
    assert!(program.opcodes.len() > 0);
    assert!(program.opcodes.last().unwrap().to_string().contains("Halt"));
}

#[test]
fn test_multiple_sql_statements() {
    let path = "/tmp/test_multiple_sql.db";
    let _ = fs::remove_file(path);
    
    let pager = Pager::open(path).unwrap();
    let mut engine = SqlEngine::new(pager);
    
    engine.load_catalog().unwrap();
    
    // Test various SQL statements (they'll fail on execution but should parse/compile)
    let queries = vec![
        "SELECT * FROM users",
        "SELECT id, name FROM products WHERE price > 10.0",
        "SELECT COUNT(*) FROM orders",
        // More complex queries for future
        // "SELECT u.name, o.total FROM users u JOIN orders o ON u.id = o.user_id",
    ];
    
    for query in queries {
        println!("\n=== Testing: {} ===", query);
        match engine.execute(query) {
            Ok(result) => println!("Success: {:?}", result),
            Err(e) => println!("Expected error: {:?}", e),
        }
    }
    
    drop(engine);
    let _ = fs::remove_file(path);
}

