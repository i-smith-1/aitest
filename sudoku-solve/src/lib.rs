use sudoku_core::{Board, Digit};

/// Solve using backtracking with MRV heuristic. Returns Some(solved) or None.
pub fn solve(mut board: Board) -> Option<Board> {
    if !board.is_consistent() {
        return None;
    }
    let n = board.n;
    // bit masks for digits used in rows/cols; bit i corresponds to digit i (1..=n)
    let mut row_mask = vec![0u64; n];
    let mut col_mask = vec![0u64; n];
    for r in 0..n {
        for c in 0..n {
            let d = board.cells[r * n + c] as usize;
            if d != 0 {
                row_mask[r] |= 1u64 << d;
                col_mask[c] |= 1u64 << d;
            }
        }
    }

    fn mrv(
        n: usize,
        board: &Board,
        row_mask: &[u64],
        col_mask: &[u64],
    ) -> Option<(usize, usize, Vec<Digit>)> {
        let mut best_rc = None;
        let mut best_opts: Vec<Digit> = Vec::new();
        for r in 0..n {
            for c in 0..n {
                if board.cells[r * n + c] != 0 {
                    continue;
                }
                // allowed = digits 1..=n not present in row/col
                let used = row_mask[r] | col_mask[c];
                let mut opts = Vec::new();
                for d in 1..=n {
                    if (used & (1u64 << d)) == 0 {
                        opts.push(d as u8);
                    }
                }
                if opts.is_empty() {
                    return Some((r, c, opts));
                }
                if best_rc.is_none() || opts.len() < best_opts.len() {
                    best_rc = Some((r, c));
                    best_opts = opts;
                    if best_opts.len() == 1 {
                        return Some((r, c, best_opts));
                    }
                }
            }
        }
        best_rc.map(|(r, c)| (r, c, best_opts))
    }

    fn backtrack(n: usize, board: &mut Board, row_mask: &mut [u64], col_mask: &mut [u64]) -> bool {
        // find next
        let next = mrv(n, board, row_mask, col_mask);
        let Some((r, c, opts)) = next else {
            // no empty cells
            return true;
        };
        if opts.is_empty() {
            return false;
        }
        for d in opts {
            // try digits
            let bit = 1u64 << (d as usize);
            if (row_mask[r] & bit) != 0 || (col_mask[c] & bit) != 0 {
                continue;
            }
            board.cells[r * n + c] = d;
            row_mask[r] |= bit;
            col_mask[c] |= bit;
            if backtrack(n, board, row_mask, col_mask) {
                return true;
            }
            // undo
            board.cells[r * n + c] = 0;
            row_mask[r] &= !bit;
            col_mask[c] &= !bit;
        }
        false
    }

    if backtrack(n, &mut board, &mut row_mask, &mut col_mask) {
        Some(board)
    } else {
        None
    }
}

/// Count solutions up to `limit` (early stop when reaching limit)
pub fn count_solutions(mut board: Board, limit: usize) -> usize {
    if limit == 0 {
        return 0;
    }
    if !board.is_consistent() {
        return 0;
    }
    let n = board.n;
    let mut row_mask = vec![0u64; n];
    let mut col_mask = vec![0u64; n];
    for r in 0..n {
        for c in 0..n {
            let d = board.cells[r * n + c] as usize;
            if d != 0 {
                row_mask[r] |= 1u64 << d;
                col_mask[c] |= 1u64 << d;
            }
        }
    }
    let mut count = 0usize;

    fn mrv(
        n: usize,
        board: &Board,
        row_mask: &[u64],
        col_mask: &[u64],
    ) -> Option<(usize, usize, Vec<Digit>)> {
        let mut best_rc = None;
        let mut best_opts: Vec<Digit> = Vec::new();
        for r in 0..n {
            for c in 0..n {
                if board.cells[r * n + c] != 0 {
                    continue;
                }
                let used = row_mask[r] | col_mask[c];
                let mut opts = Vec::new();
                for d in 1..=n {
                    if (used & (1u64 << d)) == 0 {
                        opts.push(d as u8);
                    }
                }
                if opts.is_empty() {
                    return Some((r, c, opts));
                }
                if best_rc.is_none() || opts.len() < best_opts.len() {
                    best_rc = Some((r, c));
                    best_opts = opts;
                    if best_opts.len() == 1 {
                        return Some((r, c, best_opts));
                    }
                }
            }
        }
        best_rc.map(|(r, c)| (r, c, best_opts))
    }

    fn backtrack(
        n: usize,
        board: &mut Board,
        row_mask: &mut [u64],
        col_mask: &mut [u64],
        count: &mut usize,
        limit: usize,
    ) {
        if *count >= limit {
            return;
        }
        let next = mrv(n, board, row_mask, col_mask);
        let Some((r, c, opts)) = next else {
            *count += 1;
            return;
        };
        if opts.is_empty() {
            return;
        }
        for d in opts {
            if *count >= limit {
                return;
            }
            let bit = 1u64 << (d as usize);
            if (row_mask[r] & bit) != 0 || (col_mask[c] & bit) != 0 {
                continue;
            }
            board.cells[r * n + c] = d;
            row_mask[r] |= bit;
            col_mask[c] |= bit;
            backtrack(n, board, row_mask, col_mask, count, limit);
            board.cells[r * n + c] = 0;
            row_mask[r] &= !bit;
            col_mask[c] &= !bit;
            if *count >= limit {
                return;
            }
        }
    }

    backtrack(
        n,
        &mut board,
        &mut row_mask,
        &mut col_mask,
        &mut count,
        limit,
    );
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn solves_simple() {
        // 3x3 Latin puzzle
        let puzzle = ".2.\n..3\n1..";
        let b = Board::from_str(puzzle).unwrap();
        let solved = solve(b).unwrap();
        assert!(solved.is_complete());
        assert!(solved.is_consistent());
    }
}
