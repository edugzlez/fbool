use crate::functions::BooleanFunctionCmd;
use clap::Subcommand;

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Encode a boolean function and save it to a binary file for later use")]
    Encode {
        #[command(subcommand)]
        command: BooleanFunctionCmd,

        #[arg(short, long, help = "Path where the encoded function will be saved")]
        output_path: String,
    },

    #[command(
        about = "Calculate the entanglement metric for a boolean function, which measures variable interdependence"
    )]
    Entanglement {
        #[arg(
            long,
            default_value = "false",
            help = "Compute entanglement for all variable sets instead of just the global value"
        )]
        sets: bool,

        #[arg(
            long,
            default_value = "false",
            help = "Sort the results by entanglement value"
        )]
        sorted: bool,

        #[arg(long, help = "Limit output to the first N results")]
        head: Option<usize>,

        #[arg(short, long, help = "Optional path to save results as JSON")]
        output_path: Option<String>,

        #[command(subcommand)]
        command: BooleanFunctionCmd,

        #[arg(
            long,
            default_value = "false",
            help = "Use min-max entanglement calculation instead of standard"
        )]
        minmax: bool,
    },

    #[command(
        about = "Calculate the Shannon entropy of a boolean function, measuring its information content"
    )]
    Entropy {
        #[arg(
            long,
            default_value = "false",
            help = "Compute entropy for all variable sets instead of just the global value"
        )]
        sets: bool,

        #[arg(
            long,
            default_value = "false",
            help = "Sort the results by entropy value"
        )]
        sorted: bool,

        #[arg(long, help = "Limit output to the first N results")]
        head: Option<usize>,

        #[arg(short, long, help = "Optional path to save results as JSON")]
        output_path: Option<String>,

        #[command(subcommand)]
        command: BooleanFunctionCmd,
    },

    #[command(
        about = "Compute the equanimity importance measure for a boolean function, assessing variable influence"
    )]
    EquanimityImportance {
        // #[arg(
        //     long,
        //     default_value = "false",
        //     help = "Compute equanimity importance for all variable sets instead of just the global value"
        // )]
        // sets: bool,

        // #[arg(
        //     long,
        //     default_value = "false",
        //     help = "Sort the results by equanimity importance value"
        // )]
        // sorted: bool,

        // #[arg(long, help = "Limit output to the first N results")]
        // head: Option<usize>,
        // #[arg(short, long, help = "Optional path to save results as JSON")]
        // output_path: Option<String>,
        #[command(subcommand)]
        command: BooleanFunctionCmd,
    },

    #[command(
        about = "Compute sub-information values for all variable subsets of a boolean function"
    )]
    SubInfo {
        #[command(subcommand)]
        command: BooleanFunctionCmd,
    },

    #[command(about = "Run debug operations and experiments on boolean functions")]
    Debug,
}
