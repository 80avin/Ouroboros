// Library interface for testing
// This allows integration tests to access internal modules

pub mod ir;
pub mod loaders;
pub mod memory;
pub mod symbol_resolver;
pub mod tab_viewer;

// Re-export sleigh types
pub use sleigh_compile::ldef::SleighLanguage;

// Re-export commonly used types for tests
pub use memory::Memory;
pub use ir::abstract_syntax_tree::AbstractSyntaxTree;
pub use ir::high_function::HighFunction;
