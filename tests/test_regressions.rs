mod common;
use common::*;

// Regression test template
// Copy this for each bug fix and fill in details

#[test]
#[ignore] // Remove when test case is implemented
fn test_issue_overlapping_varnodes() {
    // Bug: Overlapping varnodes caused SESE generation to skip blocks
    // Fixed in: commit 5215b08
    // Reproduction: Load binary with overlapping register usage

    // TODO: Add specific test case when we have a reproducer
}

#[test]
#[ignore] // Remove when implemented
fn test_complex_condition_simplification() {
    // Bug: Complex flag-based conditions not simplified
    // Example: (n != 1) & (overflow(n-1) == (n >= 1)) should be n > 1
    // Status: Known issue, not yet fixed

    // TODO: Add test when implemented
}

#[test]
fn test_basic_decompilation_doesnt_panic() {
    // Regression: Ensure basic decompilation doesn't panic
    // This is a catch-all for crashes during decompilation

    let test_binaries = ["simple_if", "simple_loop", "function_calls", "nested_control"];

    for binary in &test_binaries {
        let result = std::panic::catch_unwind(|| {
            load_test_binary_and_decompile(binary, "x86_64")
        });

        assert!(
            result.is_ok(),
            "Decompilation panicked for binary: {}",
            binary
        );
    }
}
