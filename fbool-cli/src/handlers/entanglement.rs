use crate::functions::BooleanFunctionCmd;
use crate::utils::{
    ExecutionTimings, HasEntanglement, OutputHandler, SortingUtils, StatsUtils, Timer,
};

/// Implementation of HasEntanglement for entanglement sets from fbool
impl HasEntanglement for fbool::entanglement::EntanglementSet {
    fn get_entanglement(&self) -> usize {
        self.entanglement
    }
}

/// Handle the entanglement command - calculate entanglement metrics
pub async fn handle_entanglement(
    command: BooleanFunctionCmd,
    sets: bool,
    sorted: bool,
    head: Option<usize>,
    output_path: Option<String>,
    minmax: bool,
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
        // Compute entanglement for all variable sets
        let mut entanglement_sets = match minmax {
            true => function.minmax_entanglement_sets(),
            false => function.entanglement_sets(),
        };

        // Record computation time
        timings.computation = computation_timer.elapsed();

        // Calculate min and max entanglement values
        if let Some((min_entanglement, max_entanglement)) =
            StatsUtils::entanglement_min_max(&entanglement_sets)
        {
            // Sort if requested
            if sorted {
                SortingUtils::sort_entanglement_sets(&mut entanglement_sets);
            }

            // Handle output
            match output_path {
                Some(path) => {
                    let json = serde_json::to_string(&entanglement_sets)?;
                    std::fs::write(path, json)?;
                }
                None => {
                    OutputHandler::print_limited(&entanglement_sets, head);
                }
            }

            println!("Minimum entanglement: {min_entanglement}");
            println!("Maximum entanglement: {max_entanglement}");
        }
    } else {
        // Compute single entanglement value
        let entanglement = match minmax {
            true => function.minmax_entanglement(),
            false => function.entanglement(),
        };

        // Record computation time
        timings.computation = computation_timer.elapsed();

        println!("Entanglement: {entanglement}");
    }

    Ok(timings)
}
