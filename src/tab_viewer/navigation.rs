use egui::Ui;

use crate::{ir::expression::Expression, memory::Memory};

use super::TabSignals;

#[derive(Clone)]
pub struct NavigationView {}

impl NavigationView {
    pub fn new() -> Self {
        Self {}
    }
    pub fn draw(&mut self, ui: &mut Ui, signals: &mut TabSignals, mem: &Memory) {
        for (&addr, _func) in mem.functions.iter() {
            let name = mem
                .symbols
                .resolve_exp(&Expression::from(addr))
                .map(|s| s.name.clone())
                .unwrap_or_else(|| format!("sub_{:x}", addr.0));
            if ui.label(&name).clicked() {
                signals.request_pos(addr);
            };
        }
    }
}
