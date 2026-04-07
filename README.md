# Rust: Fundamental and Advanced Concepts

Reference repository for fundamental and advanced concepts of Rust. Each module in [`src/concepts/`](src/concepts/) contains detailed explanations, executable examples, and tests that demonstrate specific language features.

## Content

- [`memory_and_types.rs`](src/concepts/memory_and_types.rs) - Types, sizes, alignment, layouts
- [`stack_vs_heap_performance.rs`](src/concepts/stack_vs_heap_performance.rs) - Stack vs heap benchmarks
- [`lifetimes.rs`](src/concepts/lifetimes.rs) - Implicit/explicit lifetimes, bounds, 'static lifetimes 
- [`references_vs_values.rs`](src/concepts/references_vs_values.rs) - References vs values, borrowing, auto-ref in methods, Deref/DerefMut, auto-deref coercion in structs, benchmark
- [`references_vs_values_performance.rs`](src/concepts/references_vs_values_performance.rs) - References vs values benchmarks
- [`stackframes.rs`](src/concepts/stackframes.rs) - Stack frames, passing variables
- [`array_slice_vec_string.rs`](src/concepts/array_slice_vec_string.rs) - Arrays, slices, vectors, strings, and UTF-8 handling
- [`traits_concepts.rs`](src/concepts/traits_concepts.rs) - Traits, implementations, impl/dyn, object safety
- [`clousures.rs`](src/concepts/clousures.rs) - Closures: Fn, FnMut, FnOnce, captures, trait objects
- [`sync_send.rs`](src/concepts/sync_send.rs) - Concurrency: Send, Sync, thread safety
- [`pin.rs`](src/concepts/pin.rs) - Pin, Unpin, self-referential structs
- [`futures.rs`](src/concepts/futures_async.rs) - Futures, async, await, async closures.
- [`iterators.rs`](src/concepts/iterators.rs) - Iterators, adapters, lazy evaluation
- [`syntax_sugar.rs`](src/concepts/syntax_sugar.rs) - Rust syntactic sugar
- [`comparisons.rs`](src/concepts/comparisons.rs) - Eq, PartialEq, Ord, PartialOrd
- [`concurrent_patterns.rs`](src/concepts/concurrent_patterns.rs) - Concurrency patterns: atomics, mutexes, channels, work stealing, etc.
- [`error_result.rs`](src/concepts/error_result.rs) - Error handling, Result, Option, ? operator
- [`modules_demo.rs`](src/concepts/modules_demo.rs) - Module organization strategies
- [`testing_demo.rs`](src/concepts/testing_demo.rs) - Unit tests, property-based testing (proptest)
- [`estructuras_ids.rs`](src/concepts/estructuras_ids.rs) - Structures with IDs, newtype pattern


## Quick Guide: Running and Debugging Tests

This project contains multiple demos and tests in [`src/concepts/`](src/concepts/). Below is an explanation of how to run and debug them from both VS Code and the console.

## VS Code: Rust Analyzer

- Recommended extension: [`rust-analyzer`](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
- For debugging, also install [`CodeLLDB`](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb).

How to use it:
- Open any file with tests (for example [`src/concepts/array_slice_vec_string.rs`](src/concepts/array_slice_vec_string.rs)).
- Above each `#[test]` you will see CodeLens: "Run Test" and "Debug Test".
	- "Run Test" runs that individual test.
	- "Debug Test" launches it in the debugger (CodeLLDB).
- Testing Panel: open the "Testing" panel in VS Code to see all discovered tests and run them in batch or individually.

## Console: `cargo test`

Useful commands:
- List all tests:
	```bash
	cargo test -- --list
	```

- Run an individual test (for example, invalid UTF-8 slicing):
	```bash
	cargo test --lib -- concepts::array_slice_vec_string::utf8_slicing::invalid_slice_panics -- --nocapture
	```

- Disable backtrace on panics when running tests:
	```bash
	RUST_BACKTRACE=0 cargo test --lib -- concepts
	```
