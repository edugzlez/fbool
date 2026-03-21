use crate::functions::BooleanFunctionCmd;
use std::fs::File;
use std::io::Write;

/// Handle the encode command - encode a boolean function and save it to a binary file
pub async fn handle_encode(
    command: BooleanFunctionCmd,
    output_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate the boolean function
    let function = command.into_function().await;

    // Create the output file
    let mut file = File::create(&output_path)?;

    // Encode the function to binary format
    let encoded = function.encode()?;

    // Write the encoded data to the file
    file.write_all(&encoded)?;

    println!("Successfully encoded function and saved to: {output_path}");

    Ok(())
}
