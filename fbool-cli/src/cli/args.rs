use crate::cli::commands::Commands;
use clap::Parser;

/// FBool Entanglement CLI - Compute entanglement metrics for boolean functions
#[derive(Parser)]
#[command(name = "FBool entanglement")]
#[command(version = "1.0")]
#[command(author = "Eduardo González-Vaquero <edugon07@ucm.es>")]
#[command(about = "Compute entanglement metric for boolean functions")]
pub struct Cli {
    /// The command to execute
    #[command(subcommand)]
    pub command: Commands,

    /// Measure and display execution times for generation and computation
    #[arg(
        short,
        long,
        default_value = "false",
        help = "Measure and display execution times for generation and computation"
    )]
    pub timeit: bool,
}
