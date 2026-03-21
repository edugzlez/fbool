mod cli;
mod functions;
mod handlers;
mod traits;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use handlers::{handle_debug, handle_encode, handle_entanglement, handle_entropy, handle_subinfo};

use crate::handlers::handle_equanimity_importance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let timings = match cli.command {
        Commands::Debug => {
            handle_debug().await;
            utils::ExecutionTimings::new()
        }
        Commands::Encode {
            command,
            output_path,
        } => {
            handle_encode(command, output_path).await?;
            utils::ExecutionTimings::new()
        }
        Commands::SubInfo { command } => {
            handle_subinfo(command).await?;
            utils::ExecutionTimings::new()
        }
        Commands::Entropy {
            command,
            sets,
            sorted,
            head,
            output_path,
        } => handle_entropy(command, sets, sorted, head, output_path, cli.timeit).await?,
        Commands::Entanglement {
            command,
            sets,
            output_path,
            sorted,
            head,
            minmax,
        } => {
            handle_entanglement(command, sets, sorted, head, output_path, minmax, cli.timeit)
                .await?
        }
        Commands::EquanimityImportance {
            // sets,
            // sorted,
            // head,
            // output_path,
            command,
        } => handle_equanimity_importance(command, cli.timeit).await?,
    };

    if cli.timeit {
        timings.print_if_available();
    }

    Ok(())
}
