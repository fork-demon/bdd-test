//! Executor for compiled rule templates
//!
//! Provides both WASM-sandboxed execution (recommended for security)
//! and QuickJS-based execution (for development/testing).

pub mod error;
pub mod executor;
pub mod wasm_executor;

pub use error::ExecutorError;
pub use executor::RuleExecutor;
pub use wasm_executor::{WasmExecutor, WasmLimits};
