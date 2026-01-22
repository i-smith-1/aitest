use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("dimension n must be > 0")]
    BadDim,
    #[error("parse error: {0}")]
    Parse(String),
    #[error("index out of bounds: ({r},{c}) for n={n}")]
    Oob { r: usize, c: usize, n: usize },
    #[error("digit out of range: {d} for n={n}")]
    BadDigit { d: u32, n: usize },
    #[error("json io error: {0}")]
    Json(String),
    #[error("csv parse error: {0}")]
    Csv(String),
}

/// Digit 0..=n, where 0 represents empty. Supports n > 9.
pub type Digit = u32;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Board {
    pub n: usize,
    pub cells: Vec<Digit>, // row-major, length n*n
}

impl Board {
    pub fn new(n: usize) -> Result<Self, CoreError> {
        if n == 0 {
            return Err(CoreError::BadDim);
        }
        Ok(Self {
            n,
            cells: vec![0; n * n],
        })
    }

    pub fn from_rows(rows: &[&str]) -> Result<Self, CoreError> {
        let n = rows.len();
        if n == 0 {
            return Err(CoreError::BadDim);
        }
        let mut cells = Vec::with_capacity(n * n);
        for (r, row) in rows.iter().enumerate() {
            let has_ws = row.split_whitespace().count() > 1;
            if has_ws {
                let tokens: Vec<&str> = row.split_whitespace().collect();
                if tokens.len() != n {
                    return Err(CoreError::Parse(format!(
                        "row {r} has {} tokens, expected {n}",
                        tokens.len()
                    )));
                }
                for tok in tokens {
                    let d: Digit = match tok {
                        "." => 0,
                        "0" => 0,
                        t => t
                            .parse::<u32>()
                            .map_err(|_| CoreError::Parse(format!("bad token '{t}'")))?,
                    };
                    if d as usize > n {
                        return Err(CoreError::BadDigit { d, n });
                    }
                    cells.push(d);
                }
            } else {
                // Legacy single-character-per-cell format
                let chars: Vec<char> = row.chars().collect();
                if chars.len() != n {
                    return Err(CoreError::Parse(format!(
                        "row {r} has length {}, expected {n}",
                        chars.len()
                    )));
                }
                for ch in chars {
                    let d: Digit = match ch {
                        '.' => 0,
                        '0' => 0,
                        '1'..='9' => ch.to_digit(10).unwrap() as u32,
                        _ => return Err(CoreError::Parse(format!("bad char '{ch}'"))),
                    };
                    if d as usize > n {
                        return Err(CoreError::BadDigit { d, n });
                    }
                    cells.push(d);
                }
            }
        }
        Ok(Self { n, cells })
    }

    #[inline]
    pub fn idx(&self, r: usize, c: usize) -> Result<usize, CoreError> {
        if r >= self.n || c >= self.n {
            return Err(CoreError::Oob { r, c, n: self.n });
        }
        Ok(r * self.n + c)
    }

    #[inline]
    pub fn get(&self, r: usize, c: usize) -> Result<Digit, CoreError> {
        let i = self.idx(r, c)?;
        Ok(self.cells[i])
    }

    #[inline]
    pub fn set(&mut self, r: usize, c: usize, d: Digit) -> Result<(), CoreError> {
        if d as usize > self.n {
            return Err(CoreError::BadDigit { d, n: self.n });
        }
        let i = self.idx(r, c)?;
        self.cells[i] = d;
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        !self.cells.iter().any(|&d| d == 0)
    }

    pub fn is_consistent(&self) -> bool {
        let n = self.n;
        // Check rows
        for r in 0..n {
            let mut seen = vec![false; n + 1];
            for c in 0..n {
                let d = self.cells[r * n + c] as usize;
                if d == 0 {
                    continue;
                }
                if seen[d] {
                    return false;
                }
                seen[d] = true;
            }
        }
        // Check columns
        for c in 0..n {
            let mut seen = vec![false; n + 1];
            for r in 0..n {
                let d = self.cells[r * n + c] as usize;
                if d == 0 {
                    continue;
                }
                if seen[d] {
                    return false;
                }
                seen[d] = true;
            }
        }
        true
    }

    /// Check if digit d (1..=n) can be placed at (r,c) under row/col uniqueness
    pub fn is_valid_digit(&self, r: usize, c: usize, d: Digit) -> Result<bool, CoreError> {
        if d == 0 || d as usize > self.n {
            return Err(CoreError::BadDigit { d, n: self.n });
        }
        let n = self.n;
        // row check
        for cc in 0..n {
            if cc == c {
                continue;
            }
            if self.cells[r * n + cc] == d {
                return Ok(false);
            }
        }
        // col check
        for rr in 0..n {
            if rr == r {
                continue;
            }
            if self.cells[rr * n + c] == d {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Return allowed digits for (r,c). If cell is non-empty, returns empty vec.
    pub fn allowed_digits(&self, r: usize, c: usize) -> Result<Vec<Digit>, CoreError> {
        if self.get(r, c)? != 0 {
            return Ok(vec![]);
        }
        let mut used = vec![false; self.n + 1];
        for cc in 0..self.n {
            let d = self.cells[r * self.n + cc] as usize;
            if d != 0 {
                used[d] = true;
            }
        }
        for rr in 0..self.n {
            let d = self.cells[rr * self.n + c] as usize;
            if d != 0 {
                used[d] = true;
            }
        }
        let mut out = Vec::new();
        for d in 1..=self.n {
            if !used[d] {
                out.push(d as u32);
            }
        }
        Ok(out)
    }

    /// Pretty string with rows separated by newlines; '.' for 0. Uses spaces between cells to support multi-digit values.
    pub fn to_pretty_string(&self) -> String {
        let mut s = String::new();
        for r in 0..self.n {
            for c in 0..self.n {
                if c > 0 {
                    s.push(' ');
                }
                let d = self.cells[r * self.n + c];
                if d == 0 {
                    s.push('.');
                } else {
                    s.push_str(&d.to_string());
                }
            }
            if r + 1 != self.n {
                s.push('\n');
            }
        }
        s
    }

    /// Serialize to JSON: {"n": n, "grid": [[...], ...]}
    pub fn to_json_string(&self) -> String {
        #[derive(Serialize)]
        struct B {
            n: usize,
            grid: Vec<Vec<Digit>>,
        }
        let mut grid = Vec::with_capacity(self.n);
        for r in 0..self.n {
            let mut row = Vec::with_capacity(self.n);
            for c in 0..self.n {
                row.push(self.cells[r * self.n + c]);
            }
            grid.push(row);
        }
        serde_json::to_string(&B { n: self.n, grid }).unwrap()
    }

    /// Deserialize from JSON. Accepts either {"n": n, "grid": [[numbers]]} or legacy {"n": n, "rows": ["..."]}
    pub fn from_json_str(s: &str) -> Result<Self, CoreError> {
        #[derive(Deserialize)]
        struct BGrid {
            n: usize,
            grid: Vec<Vec<Digit>>,
        }
        if let Ok(bg) = serde_json::from_str::<BGrid>(s) {
            if bg.grid.len() != bg.n || bg.grid.iter().any(|row| row.len() != bg.n) {
                return Err(CoreError::Parse("grid must be n x n".into()));
            }
            let mut b = Board::new(bg.n)?;
            for r in 0..bg.n {
                for c in 0..bg.n {
                    let d = bg.grid[r][c];
                    if d as usize > bg.n {
                        return Err(CoreError::BadDigit { d, n: bg.n });
                    }
                    b.cells[r * bg.n + c] = d;
                }
            }
            return Ok(b);
        }
        #[derive(Deserialize)]
        struct BRows {
            n: usize,
            rows: Vec<String>,
        }
        let br: BRows = serde_json::from_str(s).map_err(|e| CoreError::Json(e.to_string()))?;
        if br.rows.len() != br.n {
            return Err(CoreError::Parse("rows length != n".into()));
        }
        let rows_ref: Vec<&str> = br.rows.iter().map(|x| x.as_str()).collect();
        let board = Board::from_rows(&rows_ref)?;
        if board.n != br.n {
            return Err(CoreError::Parse("n mismatch".into()));
        }
        Ok(board)
    }

    /// Serialize to CSV with commas; 0/empty rendered as '.'
    pub fn to_csv_string(&self) -> String {
        let mut out = String::new();
        for r in 0..self.n {
            for c in 0..self.n {
                let d = self.cells[r * self.n + c];
                if c > 0 {
                    out.push(',');
                }
                match d {
                    0 => out.push('.'),
                    x => out.push_str(&x.to_string()),
                }
            }
            if r + 1 != self.n {
                out.push('\n');
            }
        }
        out
    }

    /// Parse CSV with commas; allows '.', '0' as empty; trims whitespace.
    pub fn from_csv_str(s: &str) -> Result<Self, CoreError> {
        let lines: Vec<&str> = s
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        let n = lines.len();
        if n == 0 {
            return Err(CoreError::BadDim);
        }
        let mut cells = Vec::with_capacity(n * n);
        for (r, line) in lines.iter().enumerate() {
            let cols: Vec<&str> = line.split(',').map(|t| t.trim()).collect();
            if cols.len() != n {
                return Err(CoreError::Csv(format!(
                    "row {r} has {} cols, expected {n}",
                    cols.len()
                )));
            }
            for tok in cols {
                let d: Digit = match tok {
                    "." => 0,
                    "" => 0,
                    t => t
                        .parse::<u32>()
                        .map_err(|_| CoreError::Csv(format!("bad token '{t}'")))?,
                };
                if d as usize > n {
                    return Err(CoreError::BadDigit { d, n });
                }
                cells.push(d);
            }
        }
        Ok(Self { n, cells })
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_pretty_string())
    }
}

impl FromStr for Board {
    type Err = CoreError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows: Vec<&str> = s
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        Self::from_rows(&rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_print_roundtrip() {
        let b = Board::from_rows(&["1 . 3", ". 2 .", ". . 1"]).unwrap();
        assert_eq!(b.n, 3);
        assert_eq!(b.to_string(), "1 . 3\n. 2 .\n. . 1");
        assert!(b.is_consistent());
    }

    #[test]
    fn allowed_basic() {
        let b = Board::from_rows(&["1.3", ".2.", "..1"]).unwrap();
        let a = b.allowed_digits(1, 0).unwrap();
        // row 1 has 2; col 0 has 1; allowed is {3}
        assert_eq!(a, vec![3]);
    }
}
