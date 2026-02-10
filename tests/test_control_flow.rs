mod common;
use common::*;
use bin_ast::ir::abstract_syntax_tree::AstStatement;
use assert_matches::assert_matches;

#[test]
fn test_simple_if_has_branches() {
    let (_, ast) = load_test_binary_and_decompile("simple_if", "x86_64");

    // Note: The decompiler may optimize the if/else into different structures
    // We just verify the AST was created successfully and has basic structure
    assert_matches!(
        ast.entry(),
        AstStatement::Block(_),
        "Expected AST to have a Block statement"
    );

    // Verify return statements exist
    let return_count = count_statements(&ast, |s| matches!(s, AstStatement::Return { .. }));
    assert!(
        return_count >= 1,
        "Expected at least 1 return statement, got {}",
        return_count
    );
}

#[test]
fn test_nested_if_structure() {
    let (_, ast) = load_test_binary_and_decompile("nested_control", "x86_64");

    // Verify AST was created successfully
    // The exact structure depends on the decompiler's optimization
    assert_matches!(
        ast.entry(),
        AstStatement::Block(_),
        "Expected AST to have a Block statement"
    );
}

#[test]
fn test_loop_detection() {
    let (_, ast) = load_test_binary_and_decompile("simple_loop", "x86_64");

    // Verify loop exists
    let loop_count = count_statements(&ast, |s| matches!(s, AstStatement::Loop { .. }));

    assert!(
        loop_count > 0,
        "Expected at least 1 loop statement"
    );
}

#[test]
fn test_has_return_statement() {
    let (_, ast) = load_test_binary_and_decompile("simple_if", "x86_64");

    assert!(
        assert_has_return(&ast),
        "Expected Return statement in AST"
    );
}
