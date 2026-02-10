use bin_ast::ir::abstract_syntax_tree::{AbstractSyntaxTree, AstStatement};

/// Recursively visits all AST statements and checks if the predicate matches any statement
fn visit_ast<F>(stmt: &AstStatement, predicate: &mut F) -> bool
where
    F: FnMut(&AstStatement) -> bool,
{
    if predicate(stmt) {
        return true;
    }

    match stmt {
        AstStatement::Block(block) => {
            for child in block {
                if visit_ast(child, predicate) {
                    return true;
                }
            }
        }
        AstStatement::If {
            true_statement,
            else_statement,
            ..
        } => {
            if visit_ast(true_statement, predicate) {
                return true;
            }
            if visit_ast(else_statement, predicate) {
                return true;
            }
        }
        AstStatement::Loop { body, .. } => {
            if visit_ast(body, predicate) {
                return true;
            }
        }
        AstStatement::Function { body, .. } => {
            if visit_ast(body, predicate) {
                return true;
            }
        }
        _ => {}
    }

    false
}

/// Recursively counts statements matching the predicate
fn count_statements_recursive<F>(stmt: &AstStatement, predicate: &F, count: &mut usize)
where
    F: Fn(&AstStatement) -> bool,
{
    if predicate(stmt) {
        *count += 1;
    }

    match stmt {
        AstStatement::Block(block) => {
            for child in block {
                count_statements_recursive(child, predicate, count);
            }
        }
        AstStatement::If {
            true_statement,
            else_statement,
            ..
        } => {
            count_statements_recursive(true_statement, predicate, count);
            count_statements_recursive(else_statement, predicate, count);
        }
        AstStatement::Loop { body, .. } => {
            count_statements_recursive(body, predicate, count);
        }
        AstStatement::Function { body, .. } => {
            count_statements_recursive(body, predicate, count);
        }
        _ => {}
    }
}

/// Checks if the AST contains an If statement
pub fn assert_has_if(ast: &AbstractSyntaxTree) -> bool {
    visit_ast(ast.entry(), &mut |stmt| {
        matches!(stmt, AstStatement::If { .. })
    })
}

/// Checks if the AST contains a Return statement
pub fn assert_has_return(ast: &AbstractSyntaxTree) -> bool {
    visit_ast(ast.entry(), &mut |stmt| {
        matches!(stmt, AstStatement::Return { .. })
    })
}

/// Checks if the AST contains a Loop statement
pub fn assert_has_loop(ast: &AbstractSyntaxTree) -> bool {
    visit_ast(ast.entry(), &mut |stmt| {
        matches!(stmt, AstStatement::Loop { .. })
    })
}


/// Checks if the AST contains a Call statement
pub fn assert_has_call(ast: &AbstractSyntaxTree) -> bool {
    visit_ast(ast.entry(), &mut |stmt| {
        matches!(stmt, AstStatement::Call { .. })
    })
}

/// Counts statements in the AST that match the predicate
pub fn count_statements<F>(ast: &AbstractSyntaxTree, predicate: F) -> usize
where
    F: Fn(&AstStatement) -> bool,
{
    let mut count = 0;
    count_statements_recursive(ast.entry(), &predicate, &mut count);
    count
}
