use bin_ast::memory::Memory;
use bin_ast::ir::abstract_syntax_tree::AbstractSyntaxTree;
use bin_ast::ir::high_function::HighFunction;
use bin_ast::ir::address::Address;
use bin_ast::loaders;
use bin_ast::tab_viewer::TabSignals;
use std::path::PathBuf;
use super::test_lang::create_test_memory_for_arch;

/// Loads a test binary and decompiles the entry point function
///
/// # Arguments
/// * `fixture_name` - Name of the binary file (without extension)
/// * `arch` - Architecture ("x86_32" or "x86_64")
///
/// # Returns
/// Tuple of (Memory, AbstractSyntaxTree) for the entry point function
pub fn load_test_binary_and_decompile(
    fixture_name: &str,
    arch: &str,
) -> (Memory, AbstractSyntaxTree) {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_fixtures");
    path.push(arch);
    path.push(fixture_name);

    let mut memory = create_test_memory_for_arch(arch);
    let mut signals = TabSignals::new();

    loaders::load(&path, &mut memory, &mut signals)
        .expect(&format!("Failed to load test binary: {:?}", path));

    // Get the entry point from signals
    let func_addr = get_entry_point_from_signals(&signals);

    // Mark instructions at the entry point so IR can be generated
    mark_instructions(func_addr, &mut memory);

    let hf = HighFunction::from_mem(func_addr, &memory);
    let ast = AbstractSyntaxTree::new(&hf, &memory);

    (memory, ast)
}

/// Extracts the entry point address from TabSignals
fn get_entry_point_from_signals(signals: &TabSignals) -> Address {
    use bin_ast::tab_viewer::SignalKind;

    for signal in &signals.new_signals {
        if let SignalKind::DefineFunctionStart(addr) = signal {
            return *addr;
        }
    }

    panic!("No entry point found in signals");
}

/// Marks instructions at the given address for analysis
/// This is similar to the mark_instructions function in main.rs
fn mark_instructions(addr: Address, memory: &mut Memory) {
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
                let addr: Address = instructions
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


/// Loads a test binary and returns just the memory
/// Useful for tests that need to inspect multiple functions
pub fn load_test_binary(fixture_name: &str, arch: &str) -> Memory {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_fixtures");
    path.push(arch);
    path.push(fixture_name);

    let mut memory = create_test_memory_for_arch(arch);
    let mut signals = TabSignals::new();

    loaders::load(&path, &mut memory, &mut signals)
        .expect(&format!("Failed to load test binary: {:?}", path));

    memory
}

/// Decompiles a specific function at the given address
pub fn decompile_function(memory: &Memory, func_addr: Address) -> AbstractSyntaxTree {
    let hf = HighFunction::from_mem(func_addr, memory);
    AbstractSyntaxTree::new(&hf, memory)
}
