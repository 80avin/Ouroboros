use bin_ast::SleighLanguage;
use bin_ast::memory::Memory;

// Build a fresh SleighLanguage for each test
// Note: This takes ~100ms per call, but SleighLanguage doesn't implement Clone
// and can't be shared across threads easily
fn build_x86_64_language() -> SleighLanguage {
    sleigh_compile::SleighLanguageBuilder::new(
        "./SLEIGH/Processors/x86/data/languages/x86.ldefs",
        "x86:LE:64:default",
    )
    .build()
    .expect("Failed to build x86-64 SLEIGH language")
}

fn build_x86_32_language() -> SleighLanguage {
    sleigh_compile::SleighLanguageBuilder::new(
        "./SLEIGH/Processors/x86/data/languages/x86.ldefs",
        "x86:LE:32:default",
    )
    .build()
    .expect("Failed to build x86-32 SLEIGH language")
}

pub fn create_test_memory_x86_64() -> Memory {
    Memory::new(build_x86_64_language())
}

pub fn create_test_memory_x86_32() -> Memory {
    Memory::new(build_x86_32_language())
}

pub fn create_test_memory_for_arch(arch: &str) -> Memory {
    match arch {
        "x86_64" => create_test_memory_x86_64(),
        "x86_32" => create_test_memory_x86_32(),
        _ => panic!("Unsupported architecture: {}", arch),
    }
}
