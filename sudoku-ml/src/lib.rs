//! Minimal scaffold for a future Transformer-based solver.
//! This crate intentionally does not depend on any ML framework by default
//! to keep workspace builds light. Enable the `burn` feature later to add
//! a Burn-based implementation.

use sudoku_core::Board;

/// Placeholder trait for a model that predicts digits for empty cells.
pub trait Model {
    /// Given a puzzle board, return a vector of length n*n with predicted digits (0 if undecided).
    fn predict(&self, board: &Board) -> Vec<u8>;
}

/// Placeholder solver that just returns the input.
pub struct NoopModel;

impl Model for NoopModel {
    fn predict(&self, board: &Board) -> Vec<u8> {
        board.cells.clone()
    }
}
