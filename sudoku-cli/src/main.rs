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
    /// Input format (auto by extension if not set)
    #[arg(long, value_enum, default_value_t = InFormat::Auto)]
    in_format: InFormat,
    /// Method (currently only backtrack)
    #[arg(long, value_enum, default_value_t = Method::Backtrack)]
    method: Method,
    /// Path to a Burn model file (used with --method transformer)
    #[arg(long)]
    model: Option<String>,
    /// Also print solution count up to this limit (0 to skip)
    #[arg(long, default_value_t = 0)]
    count_limit: usize,
    /// Output format for the solved grid
    #[arg(long, value_enum, default_value_t = OutFormat::Text)]
    out_format: OutFormat,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Method {
    Backtrack,
    Transformer,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum InFormat {
    Auto,
    Text,
    Json,
    Csv,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutFormat {
    Text,
    Json,
    Csv,
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
    /// Output format
    #[arg(long, value_enum, default_value_t = OutFormat::Text)]
    format: OutFormat,
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
            let ext = std::path::Path::new(&args.input)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let board = match args.in_format {
                InFormat::Text => Board::from_str(&s).context("parse text board")?,
                InFormat::Json => Board::from_json_str(&s).context("parse json board")?,
                InFormat::Csv => Board::from_csv_str(&s).context("parse csv board")?,
                InFormat::Auto => match ext.as_str() {
                    "json" => Board::from_json_str(&s).context("parse json board")?,
                    "csv" => Board::from_csv_str(&s).context("parse csv board")?,
                    _ => Board::from_str(&s).context("parse text board")?,
                },
            };
            let orig = board.clone();
            match args.method {
                Method::Backtrack => {
                    if let Some(sol) = solve(board) {
                        match args.out_format {
                            OutFormat::Text => println!("Completed grid:\n{}", sol),
                            OutFormat::Json => println!("{}", sol.to_json_string()),
                            OutFormat::Csv => println!("{}", sol.to_csv_string()),
                        }
                        if args.count_limit > 0 {
                            let k = sudoku_solve::count_solutions(orig, args.count_limit);
                            eprintln!("Solutions (<= {}): {}", args.count_limit, k);
                        }
                        Ok(())
                    } else {
                        Err(anyhow!("no solution"))
                    }
                }
                Method::Transformer => {
                    if cfg!(feature = "burn") {
                        let _model_path = args.model.as_deref();
                        #[allow(unused_mut)]
                        let mut out: Option<Board> = None;
                        #[cfg(feature = "burn")]
                        {
                            out = sudoku_ml::burn_infer::solve_with_transformer(
                                board.clone(),
                                _model_path,
                            );
                        }
                        let sol = out.or_else(|| solve(board));
                        if let Some(sol) = sol {
                            match args.out_format {
                                OutFormat::Text => println!("Completed grid:\n{}", sol),
                                OutFormat::Json => println!("{}", sol.to_json_string()),
                                OutFormat::Csv => println!("{}", sol.to_csv_string()),
                            }
                            if args.count_limit > 0 {
                                let k = sudoku_solve::count_solutions(orig, args.count_limit);
                                eprintln!("Solutions (<= {}): {}", args.count_limit, k);
                            }
                            Ok(())
                        } else {
                            Err(anyhow!("no solution"))
                        }
                    } else {
                        Err(anyhow!("Transformer not enabled. Rebuild with feature: `cargo run -p sudoku-cli --features sudoku-ml/burn -- solve --method transformer --model <path>`"))
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
            match args.format {
                OutFormat::Text => println!("{}", puzzle),
                OutFormat::Json => println!("{}", puzzle.to_json_string()),
                OutFormat::Csv => println!("{}", puzzle.to_csv_string()),
            }
            Ok(())
        }
    }
}
