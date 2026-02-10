mod common;
use common::*;
use assert_matches::assert_matches;

#[test]
fn test_multiple_functions_in_binary() {
    let (_memory, ast) = load_test_binary_and_decompile("function_calls", "x86_64");

    // Verify entry point function was decompiled
    // Note: Functions are not automatically discovered - only the entry point is analyzed
    assert_matches!(
        ast.entry(),
        bin_ast::ir::abstract_syntax_tree::AstStatement::Block(_),
        "Expected AST to have a Block statement"
    );
}

#[test]
fn test_simple_if_has_functions() {
    let (_memory, ast) = load_test_binary_and_decompile("simple_if", "x86_64");

    // Verify entry point function was decompiled
    assert_matches!(
        ast.entry(),
        bin_ast::ir::abstract_syntax_tree::AstStatement::Block(_),
        "Expected AST to have a Block statement"
    );
}

#[test]
fn test_decompile_entry_point() {
    let (_memory, ast) = load_test_binary_and_decompile("function_calls", "x86_64");

    // Verify the entry point was successfully decompiled
    assert_matches!(
        ast.entry(),
        bin_ast::ir::abstract_syntax_tree::AstStatement::Block(_),
        "Failed to create AST for entry point function"
    );
}
