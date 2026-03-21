use crate::functions::BooleanFunctionCmd;
// SubInfos trait is used via the Complete trait

/// Handle the subinfo command - compute sub-information values for all variable subsets
pub async fn handle_subinfo(command: BooleanFunctionCmd) -> Result<(), Box<dyn std::error::Error>> {
    // Generate the boolean function
    let function = command.into_function().await;

    // Compute sub-information values
    let subinfos = function.sub_infos();

    // Print the results
    println!("{subinfos:?}");

    Ok(())
}
