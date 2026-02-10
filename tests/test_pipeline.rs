mod common;
use common::*;
use assert_matches::assert_matches;

#[test]
fn test_load_and_decompile_x86_64() {
    let (_memory, ast) = load_test_binary_and_decompile("simple_if", "x86_64");

    // Basic smoke test - just verify AST was created
    assert_matches!(
        ast.entry(),
        bin_ast::ir::abstract_syntax_tree::AstStatement::Block(_),
        "Expected AST to have a Block statement at entry"
    );
}

#[test]
fn test_load_simple_loop_x86_64() {
    let (_memory, ast) = load_test_binary_and_decompile("simple_loop", "x86_64");

    // Verify AST was created for loop program
    assert_matches!(
        ast.entry(),
        bin_ast::ir::abstract_syntax_tree::AstStatement::Block(_),
        "Expected AST to have a Block statement at entry"
    );
}

#[test]
fn test_load_function_calls_x86_64() {
    let (_memory, ast) = load_test_binary_and_decompile("function_calls", "x86_64");

    // Verify AST was created for program with function calls
    assert_matches!(
        ast.entry(),
        bin_ast::ir::abstract_syntax_tree::AstStatement::Block(_),
        "Expected AST to have a Block statement at entry"
    );
}

#[test]
fn test_load_nested_control_x86_64() {
    let (_memory, ast) = load_test_binary_and_decompile("nested_control", "x86_64");

    // Verify AST was created for nested control flow program
    assert_matches!(
        ast.entry(),
        bin_ast::ir::abstract_syntax_tree::AstStatement::Block(_),
        "Expected AST to have a Block statement at entry"
    );
}
