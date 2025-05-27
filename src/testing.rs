//! Testing helpers
#![cfg(test)]

use std::{
    io::{self, Write},
    sync::{Mutex, MutexGuard},
};

/// Helper function for insta settings
pub fn timestamp_filter() -> Vec<(&'static str, &'static str)> {
    vec![(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d*\.\d*Z", "[TIMESTAMP]")]
}
