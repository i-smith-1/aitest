use rand::seq::SliceRandom;
use rand::Rng;
use sudoku_core::{Board, Digit};
use sudoku_solve::{count_solutions, solve};

/// Generate a solved Latin square of size n using a base pattern and random permutations.
pub fn gen_solved<R: Rng + ?Sized>(n: usize, rng: &mut R) -> Board {
    // base Latin square: (r + c) % n + 1
    let mut b = Board::new(n).expect("n>0");
    for r in 0..n {
        for c in 0..n {
            let v = ((r + c) % n + 1) as u32;
            b.cells[r * n + c] = v;
        }
    }
    // random digit permutation
    let mut perm_digits: Vec<Digit> = (0..=n as u32).collect(); // 0 stays 0
    let mut digits: Vec<Digit> = (1..=n as u32).collect();
    digits.shuffle(rng);
    for (i, &d) in digits.iter().enumerate() {
        perm_digits[i + 1] = d;
    }
    for v in b.cells.iter_mut() {
        *v = perm_digits[*v as usize];
    }

    // random row and column permutations
    let mut rows: Vec<usize> = (0..n).collect();
    let mut cols: Vec<usize> = (0..n).collect();
    rows.shuffle(rng);
    cols.shuffle(rng);

    let mut out = Board::new(n).unwrap();
    for (rr, &r) in rows.iter().enumerate() {
        for (cc, &c) in cols.iter().enumerate() {
            out.cells[rr * n + cc] = b.cells[r * n + c];
        }
    }
    out
}

/// Create a puzzle by removing cells down to the requested clue count.
/// If `enforce_unique` is true, ensures unique solution via counting up to 2.
pub fn make_puzzle_clues<R: Rng + ?Sized>(
    solved: &Board,
    clues: usize,
    enforce_unique: bool,
    rng: &mut R,
) -> Board {
    let n2 = solved.n * solved.n;
    let target_clues = clues.min(n2);
    let mut puzzle = solved.clone();
    let mut idxs: Vec<usize> = (0..n2).collect();
    idxs.shuffle(rng);

    // Remove cells as long as we have more than target_clues and constraints are met
    for i in idxs {
        if puzzle.cells.iter().filter(|&&d| d != 0).count() <= target_clues {
            break;
        }
        let prev = puzzle.cells[i];
        puzzle.cells[i] = 0;
        // Check solvable and uniqueness if requested
        if !puzzle.is_consistent() {
            puzzle.cells[i] = prev;
            continue;
        }
        if enforce_unique {
            let count = count_solutions(puzzle.clone(), 2);
            if count != 1 {
                puzzle.cells[i] = prev;
            }
        } else {
            if solve(puzzle.clone()).is_none() {
                puzzle.cells[i] = prev;
            }
        }
    }
    puzzle
}

pub fn make_puzzle_ratio<R: Rng + ?Sized>(
    solved: &Board,
    ratio: f32,
    enforce_unique: bool,
    rng: &mut R,
) -> Board {
    let n2 = (solved.n * solved.n) as f32;
    let clues = (ratio.clamp(0.0, 1.0) * n2).round() as usize;
    make_puzzle_clues(solved, clues, enforce_unique, rng)
}

/// Convenience: generate a puzzle from scratch.
pub fn generate_puzzle<R: Rng + ?Sized>(
    n: usize,
    clues: Option<usize>,
    ratio: Option<f32>,
    unique: bool,
    rng: &mut R,
) -> Board {
    let solved = gen_solved(n, rng);
    match (clues, ratio) {
        (Some(c), _) => make_puzzle_clues(&solved, c, unique, rng),
        (None, Some(r)) => make_puzzle_ratio(&solved, r, unique, rng),
        _ => make_puzzle_ratio(&solved, 0.6, unique, rng),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn gen_and_puzzle() {
        let mut rng = StdRng::seed_from_u64(42);
        let solved = gen_solved(5, &mut rng);
        assert!(solved.is_complete());
        assert!(solved.is_consistent());
        let puzzle = make_puzzle_ratio(&solved, 0.5, true, &mut rng);
        assert!(puzzle.is_consistent());
    }
}
