/// CLI Tests
/// 
/// Tests for the command-line interface

use deepsql::cli::commands::DotCommand;

#[test]
fn test_dot_command_parsing() {
    // Test .tables
    let cmd = DotCommand::parse(".tables");
    assert_eq!(cmd, Some(DotCommand::Tables));
    
    // Test .schema
    let cmd = DotCommand::parse(".schema users");
    assert_eq!(cmd, Some(DotCommand::Schema(Some("users".to_string()))));
    
    // Test .open
    let cmd = DotCommand::parse(".open test.db");
    assert_eq!(cmd, Some(DotCommand::Open("test.db".to_string())));
    
    // Test .help
    let cmd = DotCommand::parse(".help");
    assert_eq!(cmd, Some(DotCommand::Help));
    
    // Test .quit
    let cmd = DotCommand::parse(".quit");
    assert_eq!(cmd, Some(DotCommand::Quit));
    
    // Test .exit
    let cmd = DotCommand::parse(".exit");
    assert_eq!(cmd, Some(DotCommand::Exit));
    
    // Test invalid command
    let cmd = DotCommand::parse(".invalid");
    assert_eq!(cmd, None);
    
    // Test SQL (should not be a dot command)
    let cmd = DotCommand::parse("SELECT * FROM users");
    assert_eq!(cmd, None);
}

#[test]
fn test_help_text() {
    let help = DotCommand::help_text();
    assert!(help.contains("DeepSQL Shell"));
    assert!(help.contains(".tables"));
    assert!(help.contains(".schema"));
    assert!(help.contains(".open"));
    assert!(help.contains(".help"));
    assert!(help.contains(".quit"));
}

#[test]
fn test_dot_command_execute() {
    use deepsql::catalog::CatalogManager;
    
    let catalog = CatalogManager::new();
    
    // Test .tables on empty database
    let cmd = DotCommand::Tables;
    let result = cmd.execute(&catalog).unwrap();
    assert_eq!(result, "No tables found.");
    
    // Test .help
    let cmd = DotCommand::Help;
    let result = cmd.execute(&catalog).unwrap();
    assert!(result.contains("DeepSQL Shell"));
    
    // Test .quit
    let cmd = DotCommand::Quit;
    let result = cmd.execute(&catalog).unwrap();
    assert_eq!(result, "Goodbye!");
}

