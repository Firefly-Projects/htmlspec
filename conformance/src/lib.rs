//! The HTMLSpec conformance suite.
//!
//! This crate has no API. The suite lives entirely in `tests/`, because it must use
//! nothing an independent implementation could not: a helper exposed here would
//! invite a fixture to depend on this implementation's internals, and the point of
//! the suite is that it tests the contract.
//!
//! See `tests/conformance.rs` for the harness and `fixtures/conformance/` for the
//! fixtures it drives.
