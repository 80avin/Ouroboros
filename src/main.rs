mod ir;
mod loaders;
mod memory;
mod symbol_resolver;

use std::{
    fs::File,
    io::{stdout, Read, Write},
};

use egui::Key;
use smallvec::smallvec;
mod tab_viewer;

use eframe::egui;

use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};

use ir::{
    address::Address,
    expression::{Expression, ExpressionOp, InstructionSize, VariableSymbol},
    high_function::HighFunction,
    scope::VariableDefinition,
    type_system::VariableType,
};
use memory::{LiteralState, Memory};

use tab_viewer::{
    BlockGraph, Decompiler, MemoryView, NavigationView, SectionListView, SignalKind, TabKind,
    TabSignals, TabViewer,
};

struct DecompilerApp {
    memory: Memory,
    current_function: Option<Address>,
    signals: TabSignals,
    tree: DockState<TabKind>,
    buttons: [(&'static str, TabKind); 5],
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280., 768.0]),
        ..Default::default()
    };
    let lang = sleigh_compile::SleighLanguageBuilder::new(
        "./SLEIGH/Processors/x86/data/languages/x86.ldefs",
        "x86:LE:32:default",
    )
    .build()
    .unwrap();
    let mut memory = Memory::new(lang);

    eframe::run_native(
        "Ouroboros",
        options,
        Box::new(|cc| {
            let style = &cc.egui_ctx.style();
            let buttons = [
                (
                    "Sections",
                    TabKind::Sections(SectionListView::new(Vec::new())),
                ),
                (
                    "Listing",
                    TabKind::MemoryView(MemoryView::new(style, &memory)),
                ),
                ("Decompiler", TabKind::Decompiler(Decompiler::new(style))),
                ("Block Graph", TabKind::BlockGraph(BlockGraph::new())),
                ("Navigation", TabKind::Navigation(NavigationView::new())),
            ];

            let mut tree =
                DockState::new(vec![TabKind::MemoryView(MemoryView::new(style, &memory))]);
            let [_, new_node] = tree.split(
                (SurfaceIndex(0), NodeIndex(0)),
                egui_dock::Split::Right,
                0.5,
                egui_dock::Node::leaf(TabKind::Decompiler(Decompiler::new(style))),
            );

            tree.split(
                (SurfaceIndex(0), NodeIndex(0)),
                egui_dock::Split::Left,
                0.15,
                egui_dock::Node::leaf(TabKind::Sections(SectionListView::new(Vec::new()))),
            ); tree.split( (SurfaceIndex(0), NodeIndex(0)), egui_dock::Split::Below, 0.5, egui_dock::Node::leaf(TabKind::Navigation(NavigationView::new())),);

            let mut signals = TabSignals::new();

            if let Ok(path) = std::env::var("OUROBOROS_AUTOOPEN") {
                let sleigh_lang_id = loaders::load(path, &mut memory, &mut signals).unwrap();
                let lang = sleigh_compile::SleighLanguageBuilder::new(
                    "./SLEIGH/Processors/x86/data/languages/x86.ldefs",
                    &sleigh_lang_id,
                )
                .build()
                .unwrap();
                memory.set_language(lang);
            }

            Ok(Box::new(DecompilerApp {
                current_function: None,
                memory,
                tree,
                signals,
                buttons,
            }))
        }),
    )
}

impl eframe::App for DecompilerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_panel").show(ctx, |panel| {
            let menu_bar = egui::MenuBar::new().ui(panel, |ui| {
                ui.menu_button("File", |file_ui| {
                    let open = file_ui.button("Open...");
                    if open.clicked() {
                        if let Some(binary) = rfd::FileDialog::new()
                            .set_title("Open an executable file")
                            .pick_file()
                        {
                            let sleigh_lang_id =
                                loaders::load(binary, &mut self.memory, &mut self.signals).unwrap();
                            let lang = sleigh_compile::SleighLanguageBuilder::new(
                                "./SLEIGH/Processors/x86/data/languages/x86.ldefs",
                                &sleigh_lang_id,
                            )
                            .build()
                            .unwrap();
                            self.memory.set_language(lang);
                        }
                    }
                });
                // if ui.ctx().input(|i| i.key_pressed(Key::F)) {
                //     file.response.request_focus();
                //     todo!("Make sure a TextEdit isn't accepting input somewhere")
                // }
                ui.menu_button("Windows", |windows_ui| {
                    // let style = windows_ui.style();

                    for (name, tab) in &self.buttons {
                        if windows_ui.button(*name).clicked() {
                            if let Some(index) = self.tree.find_tab_from(|p| p == tab) {
                                self.tree.set_active_tab(index);
                            } else {
                                self.tree
                                    .main_surface_mut()
                                    .push_to_focused_leaf(tab.clone());
                            }
                        }
                    }
                });
            });
            menu_bar.response.ctx.enable_accesskit();
        });
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |panel| {
            panel.label("Status bar...");
        });

        self.signals.new_frame();

        let mut tab_viewer = TabViewer::new(&self.memory, self.current_function, &mut self.signals);
        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);

        let mut is_repopulate = false;
        let mut discovered_functions = Vec::new();
        for signal in &self.signals {
            use SignalKind::*;
            match signal {
                NewOpenFile | RepopulateInstructionRows => (),
                RequestPos(addr) => {
                    if self.memory.functions.contains_key(addr) {
                        self.current_function = Some(*addr)
                    }
                }
                RenameSymbol(var, name) => {
                    self.memory
                        .symbols
                        .resolve_mut(var)
                        .map(|v| v.name = name.clone())
                        .or_else(|| {
                            self.current_function
                                .and_then(|f| self.memory.ast.get_mut(&f))
                                .map(|ast| {
                                    let section = ast.scope.find_owning_section(var).unwrap();
                                    ast.scope.get_symbol_mut(section, var).unwrap().name =
                                        name.clone();
                                })
                        });
                }
                DefineFunctionStart(f) => {
                    if self.memory.ir.get_by_address(*f).is_none() {
                        is_repopulate = mark_instructions(*f, &mut self.memory);
                    }
                    let hf = HighFunction::from_mem(*f, &self.memory);

                    hf.fill_global_symbols(&mut self.memory);
                    hf.take_interval_ownership(&mut self.memory.navigation.function_span);

                    let ast = hf.build_ast(&self.memory);
                    self.memory.ast.insert(*f, ast);
                    self.memory.functions.insert(*f, hf);
                    self.current_function = Some(*f);

                    // Discover functions called from this function
                    discover_functions_from_calls(*f, &self.memory, &mut discovered_functions);
                }
                MarkInstruction(addr) => {
                    is_repopulate = mark_instructions(*addr, &mut self.memory);
                }
            }
        }

        // Queue discovered functions for analysis
        for func_addr in discovered_functions {
            self.signals.define_function(func_addr.0);
        }

        // Also do a global scan of all IR blocks to find functions that might be
        // called indirectly or from code we haven't analyzed yet
        discover_all_functions_from_ir(&self.memory, &mut self.signals);

        if is_repopulate {
            self.signals.repopulate_instruction_rows();
        }
    }
}

fn discover_functions_from_calls(func_addr: Address, memory: &Memory, discovered: &mut Vec<Address>) {
    use ir::basic_block::{DestinationKind, NextBlock};

    // Get the high function we just analyzed
    if let Some(hf) = memory.functions.get(&func_addr) {
        // Scan all blocks in this function for call instructions
        for block in hf.composed_blocks.iter_function(
            hf.composed_blocks.slot_by_address(func_addr).unwrap()
        ) {
            let composed_block = &hf.composed_blocks[block];

            if let NextBlock::Call { destination, .. } = &composed_block.next {
                if let DestinationKind::Concrete(called_addr) = destination {
                    // Check if this function has already been analyzed
                    if !memory.functions.contains_key(called_addr) && !discovered.contains(called_addr) {
                        println!("Auto-discovered function at {:#x} (called from {:#x})",
                                 called_addr.0, func_addr.0);
                        discovered.push(*called_addr);
                    }
                }
            }
        }
    }
}

fn discover_all_functions_from_ir(memory: &Memory, signals: &mut TabSignals) {
    use ir::basic_block::{DestinationKind, NextBlock};
    use ir::expression::ExpressionOp;

    // Scan all IR blocks for concrete call destinations
    for (_slot, block) in memory.ir.iter() {
        if let NextBlock::Call { destination, .. } = &block.next {
            if let DestinationKind::Concrete(called_addr) = destination {
                // Check if this function has already been analyzed
                if !memory.functions.contains_key(called_addr) {
                    println!("Global scan: discovered function at {:#x}", called_addr.0);
                    signals.define_function(called_addr.0);
                }
            }
        }

        // Also look for function pointers loaded into registers (e.g., main passed to __libc_start_main)
        // Check common parameter-passing registers for concrete addresses
        let param_regs = ["RDI", "RSI", "RDX", "RCX", "R8", "R9", "EDI", "ESI", "EDX", "ECX"];
        for reg_name in &param_regs {
            if let Some(var) = memory.lang.sleigh.get_reg(reg_name).and_then(|v| v.get_var()) {
                if let Some(expr) = block.registers.get(var) {
                    // Look for simple constant addresses (likely function pointers)
                    if let Some(ExpressionOp::Value(addr_val)) = expr.root_op() {
                        let addr = Address::from(*addr_val);
                        // Check if this address looks like it's in the code section
                        if memory.ir.get_by_address(addr).is_some() && !memory.functions.contains_key(&addr) {
                            println!("Global scan: discovered potential function pointer in {} at {:#x}", reg_name, addr.0);
                            signals.define_function(addr.0);
                        }
                    }
                }
            }
        }
    }
}

fn mark_instructions(addr: Address, memory: &mut Memory) -> bool {
    // dbg!((&addr, &memory.literal));
    let state = memory.literal.get_at_point_mut(addr).unwrap();
    // dbg!(&state);
    let mut is_repopulate = false;
    match &mut state.kind {
        memory::LiteralKind::Data(items) => {
            let offset = (addr.0 - state.addr.0) as usize;
            // dbg!(offset);
            let instructions = match LiteralState::from_machine_code(
                std::borrow::Cow::Borrowed(&items[offset..]),
                addr.0,
                &memory.lang,
            ) {
                Some(instrs) => instrs,
                None => {
                    eprintln!(
                        "ERROR: Failed to decode instructions at address {:#x}. \
                         This may indicate:\n\
                         - Corrupted or invalid machine code\n\
                         - Architecture mismatch (check SLEIGH language specification)\n\
                         - Data misidentified as code",
                        addr.0
                    );
                    return false;
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
                // println!("Segment has {} bytes left over...", left_over.len());
                let literal = LiteralState::from_bytes(state.addr, left_over);
                _ = memory.literal.remove_overlapping(literal.get_interval());
                println!("Adding left-over data: {:?}", literal.get_interval());
                memory
                    .literal
                    .insert_strict(literal.get_interval(), literal)
                    .unwrap();
                is_repopulate = true;
            }
            let remainder = tmp.split_off(consumed_size);
            if remainder.len() > 0 {
                let addr: Address = instructions
                    .get_instructions()
                    .last()
                    .and_then(|i| Some(i.inst_next))
                    .unwrap_or(addr.0)
                    .into();
                let literal = LiteralState::from_bytes(addr, remainder);
                _ = memory.literal.remove_overlapping(literal.get_interval());
                println!("Adding remainder data: {:?}", literal.get_interval());
                memory
                    .literal
                    .insert_strict(literal.get_interval(), literal)
                    .unwrap();
                is_repopulate = true;
            }
            if instructions.kind.size() > 0 {
                let bs = std::mem::take(&mut memory.ir);
                let ir = ir::lift(instructions.get_instructions(), &memory.lang, Some(bs));
                memory.ir = ir;
                println!("Adding instructions {:?}", instructions.get_interval());
                memory
                    .literal
                    .insert_strict(instructions.get_interval(), instructions)
                    .unwrap();
                is_repopulate = true;
            }
        }
        memory::LiteralKind::Instruction(_, _) => (),
    }
    is_repopulate
}
