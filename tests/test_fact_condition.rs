mod common;
use common::*;
use bin_ast::ir::abstract_syntax_tree::AstStatement;

#[test]
fn test_fact_function_condition_rendering() {
    // Load the a.out binary which contains the fact(int) function
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("a.out");

    let mut memory = create_test_memory_x86_64();
    let mut signals = bin_ast::tab_viewer::TabSignals::new();

    bin_ast::loaders::load(&path, &mut memory, &mut signals)
        .expect("Failed to load a.out");

    // Find the fact function (mangled name: _Z4facti at 0x1189)
    let fact_addr = bin_ast::ir::address::Address(0x1189);

    // Mark instructions and decompile
    mark_instructions_for_test(fact_addr, &mut memory);

    let hf = bin_ast::ir::high_function::HighFunction::from_mem(fact_addr, &memory);
    let ast = bin_ast::ir::abstract_syntax_tree::AbstractSyntaxTree::new(&hf, &memory);

    // Print the AST for debugging
    println!("AST entry: {:#?}", ast.entry());

    // The function should have an if statement with condition checking if n > 1
    // Let's verify the structure exists
    let has_if = count_statements(&ast, |s| matches!(s, AstStatement::If { .. }));

    if has_if > 0 {
        println!("Found {} if statement(s)", has_if);

        // Find and inspect the if statement condition
        visit_and_print_if_conditions(&ast);
    } else {
        println!("WARNING: No if statement found in fact function");
    }
}

fn visit_and_print_if_conditions(ast: &bin_ast::ir::abstract_syntax_tree::AbstractSyntaxTree) {
    visit_statement(ast.entry(), 0);
}

fn visit_statement(stmt: &AstStatement, depth: usize) {
    let indent = "  ".repeat(depth);

    match stmt {
        AstStatement::Function { body, .. } => {
            println!("{}Function:", indent);
            visit_statement(body, depth + 1);
        }
        AstStatement::If { condition, true_statement, else_statement, .. } => {
            println!("{}✓ If condition: {}", indent, condition);
            println!("{}  Then:", indent);
            visit_statement(true_statement, depth + 1);
            println!("{}  Else:", indent);
            visit_statement(else_statement, depth + 1);
        }
        AstStatement::Block(stmts) => {
            for s in stmts {
                visit_statement(s, depth);
            }
        }
        AstStatement::Return { result, .. } => {
            println!("{}Return: {}", indent, result);
        }
        _ => {
            println!("{}{:?}", indent, stmt);
        }
    }
}

// Helper to mark instructions (copied from fixtures.rs but public for this test)
fn mark_instructions_for_test(addr: bin_ast::ir::address::Address, memory: &mut bin_ast::memory::Memory) {
    use bin_ast::memory::{LiteralKind, LiteralState};
    use bin_ast::ir;

    let state = memory.literal.get_at_point_mut(addr).unwrap();
    match &mut state.kind {
        LiteralKind::Data(items) => {
            let offset = (addr.0 - state.addr.0) as usize;
            let instructions = match LiteralState::from_machine_code(
                std::borrow::Cow::Borrowed(&items[offset..]),
                addr.0,
                &memory.lang,
            ) {
                Some(instrs) => instrs,
                None => {
                    panic!("Failed to decode instructions at address {:#x}", addr.0);
                }
            };

            let consumed_size = instructions
                .get_instructions()
                .last()
                .and_then(|i| Some(i.inst_next - state.addr.0))
                .unwrap_or(0) as usize
                - offset;
            let mut left_over = std::mem::take(items);
            let mut tmp = left_over.split_off(offset);

            if left_over.len() > 0 {
                let literal = LiteralState::from_bytes(state.addr, left_over);
                _ = memory.literal.remove_overlapping(literal.get_interval());
                memory
                    .literal
                    .insert_strict(literal.get_interval(), literal)
                    .unwrap();
            }

            let remainder = tmp.split_off(consumed_size);
            if remainder.len() > 0 {
                let addr: bin_ast::ir::address::Address = instructions
                    .get_instructions()
                    .last()
                    .map(|last_inst| last_inst.inst_next)
                    .unwrap()
                    .into();
                let literal = LiteralState::from_bytes(addr, remainder);
                _ = memory.literal.remove_overlapping(literal.get_interval());
                memory
                    .literal
                    .insert_strict(literal.get_interval(), literal)
                    .unwrap();
            }

            // Lift instructions to IR
            if instructions.kind.size() > 0 {
                let bs = std::mem::take(&mut memory.ir);
                let ir_result = ir::lift(instructions.get_instructions(), &memory.lang, Some(bs));
                memory.ir = ir_result;
                _ = memory.literal.remove_overlapping(instructions.get_interval());
                memory
                    .literal
                    .insert_strict(instructions.get_interval(), instructions)
                    .unwrap();
            }
        }
        LiteralKind::Instruction(_, _) => {
            // Already marked as instructions
        }
    }
}
