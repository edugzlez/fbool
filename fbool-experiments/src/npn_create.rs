use clap::{Parser, ValueEnum};
use fbool::fvalue::FValue;
use fbool::optimal5::WithMinimalGates;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use polars::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputMode {
    Parquet,
    Stdout,
}

#[derive(Debug, Parser)]
#[command(name = "npn-create")]
#[command(about = "Compute NPN representatives and export counts")]
struct Cli {
    /// Number of truth tables to process.
    #[arg(long, default_value_t = 1 << 32)]
    max_funs: usize,

    /// Final output format.
    #[arg(long, value_enum, default_value_t = OutputMode::Parquet)]
    output: OutputMode,

    /// Output path when --output parquet is used.
    #[arg(long, default_value = "npn_table.parquet")]
    output_path: String,
}

fn process_fbool(x: usize) -> usize {
    let f = FValue::<bool>::from_usize(x, 5);
    let npn = f.npn_representant().unwrap_or(0);
    npn as usize
}

fn main() {
    let cli = Cli::parse();
    let pb = ProgressBar::new(cli.max_funs as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );
    let npn_map: HashMap<u32, u64> = (0..cli.max_funs)
        .into_par_iter()
        .progress_with(pb)
        .map(process_fbool)
        .fold(
            HashMap::new, // Inicializador de acumulador local por hilo
            |mut acc, npn| {
                *acc.entry(npn as u32).or_insert(0) += 1;
                acc
            },
        )
        .reduce(
            HashMap::new, // Combinador de resultados
            |mut map1, map2| {
                for (k, v) in map2 {
                    *map1.entry(k).or_insert(0) += v;
                }
                map1
            },
        );
    let mut keys: Vec<u32> = Vec::with_capacity(npn_map.len());
    let mut counts: Vec<u64> = Vec::with_capacity(npn_map.len());

    for (k, v) in npn_map {
        keys.push(k);
        counts.push(v);
    }

    let s1 = Series::new("npn_repr".into(), &keys);
    let s2 = Series::new("count".into(), &counts);

    let height = keys.len();
    let mut df = DataFrame::new(height, vec![Column::from(s1), Column::from(s2)]).unwrap();

    match cli.output {
        OutputMode::Parquet => {
            let file =
                File::create(&cli.output_path).expect("No se pudo crear el archivo de salida");
            ParquetWriter::new(file)
                .with_compression(ParquetCompression::Snappy)
                .finish(&mut df)
                .expect("No se pudo escribir el parquet");

            println!("\nArchivo '{}' guardado con exito.", cli.output_path);
        }
        OutputMode::Stdout => {
            let mut sorted: Vec<(u32, u64)> = keys.into_iter().zip(counts).collect();
            sorted.sort_unstable_by_key(|(k, _)| *k);

            println!("npn_repr,count");
            for (npn_repr, count) in sorted {
                println!("{},{}", npn_repr, count);
            }
        }
    }
}
