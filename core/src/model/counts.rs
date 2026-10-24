//! Counts of mutants by outcome.

/// How many mutants were caught, missed, timed out and unviable.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MutantCounts {
    /// Detected by a failing test.
    pub caught: u64,
    /// Survived: no test failed.
    pub missed: u64,
    /// Tests ran past the timeout.
    pub timeout: u64,
    /// Did not build.
    pub unviable: u64,
}
