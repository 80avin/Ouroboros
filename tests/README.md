# Ouroboros Test Suite

This directory contains integration tests for the Ouroboros decompiler that test the full pipeline from binary loading to AST construction, without requiring GUI dependencies.

## Structure

```
tests/
├── common/                      # Shared test utilities
│   ├── mod.rs                  # Re-exports all helpers
│   ├── test_lang.rs            # SLEIGH language creation
│   ├── ast_assertions.rs       # AST verification helpers
│   └── fixtures.rs             # Binary loading and decompilation helpers
├── test_pipeline.rs            # Full pipeline smoke tests (4 tests)
├── test_control_flow.rs        # Control flow detection tests (4 tests)
├── test_functions.rs           # Function decompilation tests (3 tests)
└── test_regressions.rs         # Known bug regression tests (1 test, 2 ignored)
```

## Test Fixtures

Pre-compiled test binaries are located in `test_fixtures/x86_64/`:
- `simple_if` - Simple if/else branching
- `simple_loop` - For loop
- `function_calls` - Multiple function calls
- `nested_control` - Nested if statements

To rebuild fixtures:
```bash
cd test_fixtures
make clean && make all
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test test_pipeline
cargo test test_control_flow
cargo test test_functions

# Run a specific test
cargo test test_load_and_decompile_x86_64

# Run with output
cargo test -- --nocapture
```

## Test Statistics

- **Total tests**: 22 passing (as of implementation)
  - 10 unit tests (src/ir/expression.rs)
  - 12 integration tests (tests/)
- **Test execution time**: ~5-6 seconds for full suite
- **Coverage**: Binary loading → IR lifting → CFG construction → AST generation

## Test Helpers

### Binary Loading
```rust
use common::*;

// Load and decompile entry point
let (memory, ast) = load_test_binary_and_decompile("simple_if", "x86_64");
```

### AST Assertions
```rust
// Check for specific statement types
assert_has_if(&ast);
assert_has_return(&ast);
assert_has_loop(&ast);

// Count statements matching a predicate
let return_count = count_statements(&ast, |s| matches!(s, AstStatement::Return { .. }));
```

## Notes

- **SLEIGH Language**: Each test builds a fresh SleighLanguage (~100ms) since it doesn't implement Clone
- **Test Cut Point**: Tests stop at AST generation, before GUI rendering (tab_viewer/decompiler.rs)
- **Entry Point Only**: Tests decompile the binary entry point, not all discovered functions
- **Optimization Aware**: Tests don't assume specific AST structures since the decompiler may optimize code

## Adding New Tests

1. **Create test fixture** (if needed):
   ```c
   // test_fixtures/my_test.c
   int my_function(int x) {
       // Your test case
   }
   ```

2. **Add to Makefile**:
   ```makefile
   SOURCES = simple_if simple_loop function_calls nested_control my_test
   ```

3. **Write integration test**:
   ```rust
   #[test]
   fn test_my_feature() {
       let (memory, ast) = load_test_binary_and_decompile("my_test", "x86_64");
       // Your assertions
   }
   ```

4. **Rebuild fixtures**:
   ```bash
   cd test_fixtures && make all
   ```

## Regression Tests

Known bugs should be documented in `test_regressions.rs`:
```rust
#[test]
#[ignore] // Remove when fixed
fn test_issue_42_description() {
    // Bug: Description of the issue
    // Fixed in: commit hash (when fixed)
    // TODO: Add specific test case
}
```

## Future Enhancements

- [ ] Add x86-32 test fixtures (requires gcc-multilib)
- [ ] Test more complex control flow (switch, break, continue)
- [ ] Test function call analysis
- [ ] Test memory and register tracking
- [ ] Performance benchmarks for decompilation speed
