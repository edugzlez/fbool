use crate::functions::BooleanFunctionCmd;
use crate::utils::{ExecutionTimings, HasEntropy, OutputHandler, SortingUtils, StatsUtils, Timer};

/// Implementation of HasEntropy for entropy sets from fbool
impl HasEntropy for fbool::entanglement::EntropySet {
    fn get_entropy(&self) -> f32 {
        self.entropy
    }
}

/// Handle the entropy command - calculate Shannon entropy metrics
pub async fn handle_entropy(
    command: BooleanFunctionCmd,
    sets: bool,
    sorted: bool,
    head: Option<usize>,
    output_path: Option<String>,
    timeit: bool,
) -> Result<ExecutionTimings, Box<dyn std::error::Error>> {
    let mut timings = ExecutionTimings::new();
    let mut generation_timer = Timer::new();
    let mut computation_timer = Timer::new();

    // Start generation timing
    generation_timer.start_if(timeit);

    // Generate the boolean function
    let function = command.into_function().await;

    // Record generation time
    timings.generation = generation_timer.elapsed();

    // Start computation timing
    computation_timer.start_if(timeit);

    if sets {
        // Compute entropy for all variable sets
        let mut entropy_sets = function.entropy_sets();

        // Record computation time
        timings.computation = computation_timer.elapsed();

        // Calculate min and max entropy values
        if let Some((min_entropy, max_entropy)) = StatsUtils::entropy_min_max(&entropy_sets) {
            // Sort if requested
            if sorted {
                SortingUtils::sort_entropy_sets(&mut entropy_sets);
            }

            // Handle output
            match output_path {
                Some(path) => {
                    let json = serde_json::to_string(&entropy_sets)?;
                    std::fs::write(path, json)?;
                }
                None => {
                    OutputHandler::print_limited(&entropy_sets, head);
                }
            }

            println!(
                "Minimum entropy(n={}): {}",
                function.num_vars(),
                min_entropy
            );
            println!(
                "Maximum entropy(n={}): {}",
                function.num_vars(),
                max_entropy
            );
        }
    } else {
        // Compute single entropy value
        let entropy = function.entropy();

        // Record computation time
        timings.computation = computation_timer.elapsed();

        println!("Entropy(n={}): {}", function.num_vars(), entropy);
    }

    Ok(timings)
}
