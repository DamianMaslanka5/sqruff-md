use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "sqruff-md")]
#[command(about = "sqruff-md is a sql formatter and linter for sql in markdown", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    /// Path to a configuration file.
    #[arg(long, global = true)]
    pub config: Option<String>,
    #[arg(long, global = true)]
    pub paths: Vec<String>,
    #[arg(long, global = true)]
    pub ignore_unparsable: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(
        name = "lint",
        about = "Lint SQL files via passing a list of files or using stdin"
    )]
    Lint,
    #[command(
        name = "fix",
        about = "Fix SQL files via passing a list of files or using stdin"
    )]
    Fix,
}
