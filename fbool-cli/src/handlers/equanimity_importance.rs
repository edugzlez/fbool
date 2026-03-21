use crate::functions::BooleanFunctionCmd;
use crate::utils::{ExecutionTimings, Timer};

/// Handle the equanimity importance command - calculate equanimity importance metrics
pub async fn handle_equanimity_importance(
    command: BooleanFunctionCmd,
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

    // Compute single equanimity importance value
    let equanimity_importance = function.equanimity_importance();

    // Record computation time
    timings.computation = computation_timer.elapsed();

    println!("Equanimity Importance: {equanimity_importance}");

    Ok(timings)
}
