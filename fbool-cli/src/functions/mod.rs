// All function implementations are now in this module

use crate::traits::Complete;
use clap::Subcommand;
use fbool::fvalue::FValue;
use std::{fs::File, io::Read};

/// All available boolean function types in a flat structure
#[derive(Subcommand)]
pub enum BooleanFunctionCmd {
    // Logic functions
    #[command(about = "Majority function")]
    Majority {
        #[arg(
            short,
            long,
            default_value = "3",
            help = "Number of input variables for the majority function"
        )]
        n_vars: usize,
    },

    #[command(about = "Parity function")]
    Parity {
        #[arg(
            short,
            long,
            default_value = "3",
            help = "Number of input variables for the parity function"
        )]
        n_vars: usize,
    },

    #[command(about = "Equality function")]
    Eq {
        #[arg(short, long, help = "Number of input variables to check for equality")]
        n_vars: usize,
    },

    #[command(about = "Ordered")]
    Ordered {
        #[arg(
            short,
            long,
            help = "Number of input variables to check if they are in ascending order"
        )]
        n_vars: usize,
    },

    // Arithmetic functions
    #[command(about = "Multiply")]
    Multiply {
        #[arg(
            short,
            long,
            help = "Number of input variables for the multiplication function"
        )]
        n_vars: usize,
    },

    #[command(about = "Sum")]
    Sum {
        #[arg(short, long, help = "Number of input variables to sum")]
        n_vars: usize,

        #[arg(
            short,
            long,
            help = "Optional limit on the number of summands to include"
        )]
        summands: Option<usize>,
    },

    #[command(about = "Maximum")]
    Max {
        #[arg(short, long, help = "Number of input variables to find the maximum of")]
        n_vars: usize,
    },

    #[command(about = "GCD")]
    Gcd {
        #[arg(short, long, help = "Number of input variables to compute the GCD of")]
        n_vars: usize,
    },

    // Crypto functions
    #[command(about = "Primality function")]
    Primality {
        #[arg(
            short,
            long,
            default_value = "3",
            help = "Number of input variables for the primality function"
        )]
        n_vars: usize,
    },

    #[command(about = "Sum is prime")]
    SumIsPrime {
        #[arg(
            short,
            long,
            help = "Number of input variables to sum and check for primality"
        )]
        n_vars: usize,
    },

    #[command(about = "Coprimes")]
    Coprimes {
        #[arg(
            short,
            long,
            help = "Number of input variables to check for coprimality"
        )]
        n_vars: usize,
    },

    // Custom functions
    #[command(about = "Constant function")]
    Constant {
        #[arg(
            short,
            long,
            default_value = "3",
            help = "Number of input variables for the constant function"
        )]
        n_vars: usize,
        #[arg(short, long, help = "The constant boolean value to return")]
        value: bool,
    },

    #[command(about = "Usize constant")]
    UsizeConstant {
        #[arg(
            short,
            long,
            help = "Number of input variables for the usize constant function"
        )]
        n_vars: usize,

        #[arg(short, long, help = "The constant usize value to return")]
        value: usize,
    },

    #[command(about = "Custom function")]
    Raw {
        #[arg(
            short,
            long,
            help = "Vector of truth values (0 or 1) defining the custom function"
        )]
        vector: Vec<u8>,
    },

    #[command(about = "Binary")]
    Bin {
        #[arg(
            short,
            long,
            help = "Path to a binary file containing a boolean function"
        )]
        path: String,
    },

    // Meta functions
    #[command(about = "Find zero function")]
    FindZero {
        #[arg(
            short,
            long,
            default_value = "3",
            help = "Size of the vector to search in"
        )]
        vector_size: usize,
        #[arg(
            short,
            long,
            default_value = "3",
            help = "Size of each element in the vector"
        )]
        element_size: usize,
    },

    #[command(about = "Meta function")]
    Meta {
        #[arg(
            short,
            long,
            help = "Number of input variables for the meta-entanglement function"
        )]
        n_vars: usize,
    },
}

impl BooleanFunctionCmd {
    pub async fn into_function(self) -> Box<dyn Complete> {
        match self {
            // Logic functions
            BooleanFunctionCmd::Majority { n_vars } => Box::new(FValue::majority(n_vars)),
            BooleanFunctionCmd::Parity { n_vars } => Box::new(FValue::parity(n_vars)),
            BooleanFunctionCmd::Eq { n_vars } => Box::new(FValue::equality(n_vars)),
            BooleanFunctionCmd::Ordered { n_vars } => Box::new(FValue::ordered(n_vars)),

            // Arithmetic functions
            BooleanFunctionCmd::Multiply { n_vars } => {
                let fvalue = FValue::<usize>::multiply(n_vars, 2usize);
                Box::new(fvalue)
            }
            BooleanFunctionCmd::Sum { n_vars, summands } => match summands {
                Some(some) => {
                    let fvalue = FValue::<usize>::sum_some(n_vars, some);
                    Box::new(fvalue)
                }
                None => {
                    let fvalue = FValue::<usize>::sum(n_vars);
                    Box::new(fvalue)
                }
            },
            BooleanFunctionCmd::Max { n_vars } => {
                let fvalue = FValue::<usize>::max(n_vars);
                Box::new(fvalue)
            }
            BooleanFunctionCmd::Gcd { n_vars } => {
                let fvalue = FValue::<usize>::gcd(n_vars);
                Box::new(fvalue)
            }

            // Crypto functions
            BooleanFunctionCmd::Primality { n_vars } => Box::new(FValue::primality(n_vars)),
            BooleanFunctionCmd::SumIsPrime { n_vars } => Box::new(FValue::sum_is_prime(n_vars)),
            BooleanFunctionCmd::Coprimes { n_vars } => Box::new(FValue::coprimes(n_vars)),

            // Custom functions
            BooleanFunctionCmd::Constant { n_vars, value } => {
                Box::new(FValue::<bool>::constant(n_vars, value))
            }
            BooleanFunctionCmd::UsizeConstant { n_vars, value } => {
                let fvalue = FValue::<usize>::constant(n_vars, value);
                Box::new(fvalue)
            }
            BooleanFunctionCmd::Raw { vector } => {
                Box::new(FValue::new(vector.into_iter().map(|x| x > 0).collect()))
            }
            BooleanFunctionCmd::Bin { path } => {
                let mut file = File::open(path).unwrap();
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).unwrap();
                let fb: FValue<bool> =
                    bincode::decode_from_slice(&buffer, bincode::config::standard())
                        .map(|(fbool, _)| fbool)
                        .unwrap();

                Box::new(fb)
            }

            // Meta functions
            BooleanFunctionCmd::FindZero {
                vector_size,
                element_size,
            } => Box::new(FValue::find_zero(vector_size, element_size)),
            BooleanFunctionCmd::Meta { n_vars } => {
                let fvalue = FValue::<usize>::meta_entanglement(n_vars).await;
                Box::new(fvalue)
            }
        }
    }
}
