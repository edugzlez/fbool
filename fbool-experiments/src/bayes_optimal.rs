use clap::Parser;
#[path = "metrics.rs"]
mod metrics;

use metrics::{MetricType, metric_columns};
use polars::prelude::*;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Bayes optimal upper bound for classification"
)]
struct Args {
    #[arg(long, default_value = "npn_table2.parquet")]
    input: String,

    #[arg(long, num_args = 1.., value_delimiter = ',')]
    metrics: Vec<String>,

    #[arg(long, default_value = "min_gates")]
    target: String,

    #[arg(long, default_value_t = 9)]
    threshold: u32,

    #[arg(long, default_value_t = false)]
    binary: bool,

    #[arg(long, default_value = "count")]
    count_col: String,

    #[arg(long)]
    output_csv: Option<String>,
}

#[derive(Debug)]
struct BayesOptimalResult {
    input_file: String,
    feature_columns: String,
    prediction_mode: String,
    target_rule: String,
    count_column: String,
    correct_majority_mass: u64,
    max_accuracy_percent: f64,
}

fn selected_feature_columns(raw_metrics: &[String]) -> PolarsResult<Vec<String>> {
    let available_cols: HashSet<String> = metric_columns(MetricType::All)
        .into_iter()
        .map(|x| x.to_string())
        .collect();

    if raw_metrics.is_empty() {
        return Ok(metric_columns(MetricType::All)
            .into_iter()
            .map(|x| x.to_string())
            .collect());
    }

    let mut cols = Vec::new();
    for col in raw_metrics {
        if !available_cols.contains(col) {
            return Err(PolarsError::ComputeError(
                format!(
                    "Unknown metric column '{}'. Available columns: {}",
                    col,
                    metric_columns(MetricType::All).join(", ")
                )
                .into(),
            ));
        }

        if !cols.iter().any(|existing| existing == col) {
            cols.push(col.clone());
        }
    }

    Ok(cols)
}

fn main() -> PolarsResult<()> {
    let args = Args::parse();

    let file = File::open(&args.input)?;
    let df = ParquetReader::new(file).finish()?;

    let feature_columns = selected_feature_columns(&args.metrics)?;

    for col in feature_columns.iter().chain([args.target.clone()].iter()) {
        if !df
            .get_column_names()
            .iter()
            .any(|name| name.as_str() == col)
        {
            return Err(PolarsError::ComputeError(
                format!("Column '{col}' not found in input parquet").into(),
            ));
        }
    }

    if !df
        .get_column_names()
        .iter()
        .any(|name| name.as_str() == args.count_col)
    {
        return Err(PolarsError::ComputeError(
            format!(
                "Count column '{}' not found in input parquet",
                args.count_col
            )
            .into(),
        ));
    }

    let total_functions: u64 = df.column(&args.count_col)?.u64()?.sum().unwrap_or(0);

    let (prepared_df, label_col, target_rule) = if args.binary {
        (
            df.lazy()
                .with_column(
                    col(&args.target)
                        .lt_eq(lit(args.threshold))
                        .alias("is_small"),
                )
                .collect()?,
            "is_small".to_string(),
            format!("{} <= {}", args.target, args.threshold),
        )
    } else {
        (df, args.target.clone(), args.target.clone())
    };

    let mut group_columns = feature_columns.clone();
    group_columns.push(label_col.clone());

    let grouped = prepared_df
        .lazy()
        .group_by([cols(group_columns.clone())])
        .agg([col(&args.count_col).sum().alias("sum_count")])
        .collect()?;

    let max_acc_df = grouped
        .lazy()
        .group_by([cols(feature_columns.clone())])
        .agg([col("sum_count").max().alias("majority_count")])
        .select([col("majority_count").sum()])
        .collect()?;

    let correct_predictions: u64 = max_acc_df
        .column("majority_count")?
        .u64()?
        .sum()
        .unwrap_or(0);

    let max_accuracy = if total_functions == 0 {
        0.0
    } else {
        (correct_predictions as f64 / total_functions as f64) * 100.0
    };

    let feature_list = if feature_columns.is_empty() {
        "(none)".to_string()
    } else {
        feature_columns.join(", ")
    };

    let result = BayesOptimalResult {
        input_file: args.input.clone(),
        feature_columns: feature_list.clone(),
        prediction_mode: if args.binary { "binary" } else { "multiclass" }.to_string(),
        target_rule,
        count_column: args.count_col.clone(),
        correct_majority_mass: correct_predictions,
        max_accuracy_percent: max_accuracy,
    };

    // Export to CSV if specified
    if let Some(output_path) = &args.output_csv {
        let mut file = std::fs::File::create(output_path).map_err(|e| {
            PolarsError::ComputeError(format!("Failed to create output file: {}", e).into())
        })?;

        writeln!(
            file,
            "input_file,features,mode,target_rule,count_column,correct_mass,max_accuracy_percent"
        )
        .map_err(|e| PolarsError::ComputeError(format!("Failed to write CSV: {}", e).into()))?;

        writeln!(
            file,
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{},{:.6}",
            result.input_file,
            result.feature_columns.replace("\"", "\"\""),
            result.prediction_mode,
            result.target_rule,
            result.count_column,
            result.correct_majority_mass,
            result.max_accuracy_percent
        )
        .map_err(|e| PolarsError::ComputeError(format!("Failed to write CSV: {}", e).into()))?;

        eprintln!("CSV output written to: {}", output_path);
    } else {
        // Print human-readable output
        println!("\n======================= Bayes Optimal Report =======================");
        println!("Input file                 : {}", result.input_file);
        println!("Feature columns            : {}", result.feature_columns);
        println!("Prediction mode            : {}", result.prediction_mode);
        println!("Target rule                : {}", result.target_rule);
        println!("Count column               : {}", result.count_column);
        println!(
            "Correct majority mass      : {}",
            result.correct_majority_mass
        );
        println!(
            "Maximum theoretical accuracy: {:.2}%",
            result.max_accuracy_percent
        );
        println!("==================================================================\n");
    }

    Ok(())
}
