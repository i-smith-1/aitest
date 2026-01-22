use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use rand::SeedableRng;
use sudoku_core::Board;
use sudoku_gen::generate_puzzle;
use sudoku_solve::solve;

#[derive(Parser)]
#[command(name = "sudoku")]
#[command(about = "Latin Sudoku (n x n) CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Solve a puzzle from a file or stdin
    Solve(SolveArgs),
    /// Generate a puzzle
    Gen(GenArgs),
}

#[derive(Args)]
struct SolveArgs {
    /// Input path (use '-' for stdin)
    #[arg(long, value_name = "PATH", default_value = "-")]
    input: String,
    /// Method (currently only backtrack)
    #[arg(long, value_enum, default_value_t = Method::Backtrack)]
    method: Method,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Method {
    Backtrack,
}

#[derive(Args)]
struct GenArgs {
    /// Grid size n
    #[arg(long)]
    n: usize,
    /// Number of clues to keep (mutually exclusive with --ratio)
    #[arg(long)]
    clues: Option<usize>,
    /// Ratio of clues to keep (0..1), ignored if --clues is set
    #[arg(long)]
    ratio: Option<f32>,
    /// Enforce unique solution
    #[arg(long, default_value_t = false)]
    unique: bool,
    /// RNG seed (optional)
    #[arg(long)]
    seed: Option<u64>,
}

fn read_input(path: &str) -> Result<String> {
    if path == "-" {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s)?;
        Ok(s)
    } else {
        Ok(fs::read_to_string(PathBuf::from(path)).with_context(|| format!("reading {path}"))?)
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Solve(args) => {
            let s = read_input(&args.input)?;
            let board = Board::from_str(&s).context("parse board")?;
            match args.method {
                Method::Backtrack => {
                    if let Some(sol) = solve(board) {
                        println!("{}", sol);
                        Ok(())
                    } else {
                        Err(anyhow!("no solution"))
                    }
                }
            }
        }
        Commands::Gen(args) => {
            let mut rng = if let Some(seed) = args.seed {
                rand::rngs::StdRng::seed_from_u64(seed)
            } else {
                rand::rngs::StdRng::from_entropy()
            };
            let puzzle = generate_puzzle(args.n, args.clues, args.ratio, args.unique, &mut rng);
            println!("{}", puzzle);
            Ok(())
        }
    }
}
