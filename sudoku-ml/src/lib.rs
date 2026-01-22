//! Minimal scaffold for a future Transformer-based solver.
//! This crate intentionally does not depend on any ML framework by default
//! to keep workspace builds light. Enable the `burn` feature later to add
//! a Burn-based implementation.

use sudoku_core::{Board, Digit};

/// Placeholder trait for a model that predicts digits for empty cells.
pub trait Model {
    /// Given a puzzle board, return a vector of length n*n with predicted digits (0 if undecided).
    fn predict(&self, board: &Board) -> Vec<Digit>;
}

/// Placeholder solver that just returns the input.
pub struct NoopModel;

impl Model for NoopModel {
    fn predict(&self, board: &Board) -> Vec<Digit> {
        board.cells.clone()
    }
}

#[cfg(feature = "burn")]
pub mod burn_infer {
    use super::*;
    // Touch burn so the feature is meaningful; keep implementation minimal for now.
    use burn::tensor::backend::Backend as _;

    /// Placeholder: future Burn-based transformer inference.
    /// Currently returns None to let the CLI fall back to classical solving.
    pub fn solve_with_transformer(_puzzle: Board, _model_path: Option<&str>) -> Option<Board> {
        None
    }
}
