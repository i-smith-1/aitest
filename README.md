Latin Sudoku (n x n) in Rust

This workspace implements a general n-by-n "Latin Sudoku" where digits 1..n must appear exactly once per row and once per column. It includes:

- Core board representation and validation
- Puzzle generator (Latin square + masking with optional uniqueness)
- Classical solver (MRV backtracking)
- CLI for generating and solving puzzles
- ML crate scaffold for a future Transformer (feature-gated)

Crates
- sudoku-core: Board, parsing/printing, constraints and helpers
- sudoku-gen: Solved Latin square generator and puzzle maker
- sudoku-solve: Backtracking solver and solution counter
- sudoku-cli: User-facing binary (solve/gen)
- sudoku-ml: Scaffold for a Burn-based Transformer (not required to build)

Rules and Representation
- Digits are `1..=n`; `0` means empty
- Board is row-major vector of length `n*n`
- Only row/column uniqueness rules are enforced (no boxes)

CLI
- Generate: `cargo run -p sudoku-cli -- gen --n 5 --ratio 0.5 --unique`
- Solve:    `cargo run -p sudoku-cli -- solve --input puzzle.txt`

Plan Snapshot
- Data/modeling (future): sequence of length n*n, tokens 0..n (0 empty). Transformer encoder with token+row+col embeddings. Mask invalid digits during inference. Train on-the-fly generated Latin puzzles.

Milestones
1) Core + Solver (implemented)
2) Generator + CLI (implemented)
3) ML scaffold (stubbed, gated)
