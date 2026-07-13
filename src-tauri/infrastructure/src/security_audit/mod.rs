//! Security audit tests for infrastructure layer.
//!
//! Covers: SQLCipher cold dump analysis, key zeroization,
//! key file permissions, PRAGMA key injection safety.

pub mod sqlcipher_tests;
