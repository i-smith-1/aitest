use sudoku_core::{Board, Digit};

/// Solve using backtracking with MRV heuristic. Returns Some(solved) or None.
pub fn solve(mut board: Board) -> Option<Board> {
    if !board.is_consistent() {
        return None;
    }
    let n = board.n;
    let mut row_used = vec![vec![false; n + 1]; n];
    let mut col_used = vec![vec![false; n + 1]; n];
    for r in 0..n {
        for c in 0..n {
            let d = board.cells[r * n + c] as usize;
            if d != 0 {
                row_used[r][d] = true;
                col_used[c][d] = true;
            }
        }
    }

    fn mrv(
        n: usize,
        board: &Board,
        row_used: &[Vec<bool>],
        col_used: &[Vec<bool>],
    ) -> Option<(usize, usize, Vec<Digit>)> {
        let mut best: Option<(usize, usize, Vec<Digit>)> = None;
        for r in 0..n {
            for c in 0..n {
                if board.cells[r * n + c] != 0 {
                    continue;
                }
                let mut opts: Vec<Digit> = Vec::new();
                for d in 1..=n {
                    if !row_used[r][d] && !col_used[c][d] {
                        opts.push(d as Digit);
                    }
                }
                if opts.is_empty() {
                    return Some((r, c, opts));
                }
                if best.as_ref().map(|(_, _, v)| v.len()).unwrap_or(usize::MAX) > opts.len() {
                    best = Some((r, c, opts));
                    if let Some((_, _, ref v)) = best {
                        if v.len() == 1 {
                            return best;
                        }
                    }
                }
            }
        }
        best
    }

    fn backtrack(
        n: usize,
        board: &mut Board,
        row_used: &mut [Vec<bool>],
        col_used: &mut [Vec<bool>],
    ) -> bool {
        let next = mrv(n, board, row_used, col_used);
        let Some((r, c, opts)) = next else {
            return true;
        };
        if opts.is_empty() {
            return false;
        }
        for d in opts {
            let du = d as usize;
            if row_used[r][du] || col_used[c][du] {
                continue;
            }
            board.cells[r * n + c] = d;
            row_used[r][du] = true;
            col_used[c][du] = true;
            if backtrack(n, board, row_used, col_used) {
                return true;
            }
            board.cells[r * n + c] = 0;
            row_used[r][du] = false;
            col_used[c][du] = false;
        }
        false
    }

    if backtrack(n, &mut board, &mut row_used, &mut col_used) {
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
    let mut row_used = vec![vec![false; n + 1]; n];
    let mut col_used = vec![vec![false; n + 1]; n];
    for r in 0..n {
        for c in 0..n {
            let d = board.cells[r * n + c] as usize;
            if d != 0 {
                row_used[r][d] = true;
                col_used[c][d] = true;
            }
        }
    }
    let mut count = 0usize;

    fn mrv(
        n: usize,
        board: &Board,
        row_used: &[Vec<bool>],
        col_used: &[Vec<bool>],
    ) -> Option<(usize, usize, Vec<Digit>)> {
        let mut best: Option<(usize, usize, Vec<Digit>)> = None;
        for r in 0..n {
            for c in 0..n {
                if board.cells[r * n + c] != 0 {
                    continue;
                }
                let mut opts: Vec<Digit> = Vec::new();
                for d in 1..=n {
                    if !row_used[r][d] && !col_used[c][d] {
                        opts.push(d as Digit);
                    }
                }
                if opts.is_empty() {
                    return Some((r, c, opts));
                }
                if best.as_ref().map(|(_, _, v)| v.len()).unwrap_or(usize::MAX) > opts.len() {
                    best = Some((r, c, opts));
                    if let Some((_, _, ref v)) = best {
                        if v.len() == 1 {
                            return best;
                        }
                    }
                }
            }
        }
        best
    }

    fn backtrack(
        n: usize,
        board: &mut Board,
        row_used: &mut [Vec<bool>],
        col_used: &mut [Vec<bool>],
        count: &mut usize,
        limit: usize,
    ) {
        if *count >= limit {
            return;
        }
        let next = mrv(n, board, row_used, col_used);
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
            let du = d as usize;
            if row_used[r][du] || col_used[c][du] {
                continue;
            }
            board.cells[r * n + c] = d;
            row_used[r][du] = true;
            col_used[c][du] = true;
            backtrack(n, board, row_used, col_used, count, limit);
            board.cells[r * n + c] = 0;
            row_used[r][du] = false;
            col_used[c][du] = false;
            if *count >= limit {
                return;
            }
        }
    }

    backtrack(
        n,
        &mut board,
        &mut row_used,
        &mut col_used,
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
