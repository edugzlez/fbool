use clap::Parser;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use indicatif::{ProgressBar, ProgressStyle};
#[path = "metrics.rs"]
mod metrics;
use polars::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use metrics::{MetricType, metric_columns};

#[derive(Debug, Clone)]
struct PairReport {
    metric_a: String,
    metric_b: String,
    i_y_a: f64,
    i_y_b: f64,
    i_y_ab: f64,
    coinformation: f64,
    kappa: Option<f64>,
}

#[derive(Debug, Clone)]
struct EntropyCache {
    values: HashMap<Vec<String>, f64>,
}

impl EntropyCache {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    fn entropy(
        &mut self,
        df: &DataFrame,
        columns: &[String],
        weight_column: Option<&str>,
    ) -> PolarsResult<f64> {
        let mut key = columns.to_vec();
        key.sort_unstable();
        key.dedup();

        if let Some(value) = self.values.get(&key) {
            return Ok(*value);
        }

        let value = entropy_for_columns(df, &key, weight_column)?;
        self.values.insert(key, value);
        Ok(value)
    }
}

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Mutual information, coinformation, and orthogonality diagnostics"
)]
struct Args {
    /// Input parquet file.
    #[arg(long, default_value = "npn_table2.parquet")]
    input: String,

    /// Target column to analyze.
    #[arg(long, default_value = "min_gates")]
    target: String,

    /// Optional binary target threshold. When enabled, the target becomes target <= threshold.
    #[arg(long)]
    binary: bool,

    /// Threshold used when --binary is enabled.
    #[arg(long, default_value_t = 9)]
    threshold: u32,

    /// Count column used as row weight.
    #[arg(long, default_value = "count")]
    count_column: String,

    /// Metric columns to analyze. If omitted, all columns except target/count are used.
    #[arg(long, num_args = 1.., value_delimiter = ',')]
    metrics: Vec<String>,

    /// Optional CSV output file for pair reports
    #[arg(long)]
    output_csv: Option<String>,

    /// Optional CSV output file for mutual information summary
    #[arg(long)]
    output_mi_csv: Option<String>,
}

fn ensure_columns_exist(df: &DataFrame, columns: &[String]) -> PolarsResult<()> {
    let existing = df.get_column_names();
    for column in columns {
        if !existing.iter().any(|name| name.as_str() == column) {
            return Err(PolarsError::ComputeError(
                format!("Column '{column}' not found in input parquet").into(),
            ));
        }
    }

    Ok(())
}

fn total_weight(df: &DataFrame, weight_column: Option<&str>) -> PolarsResult<f64> {
    match weight_column {
        Some(column) => {
            let series = df.column(column)?.cast(&DataType::Float64)?;
            Ok(series.f64()?.sum().unwrap_or(0.0))
        }
        None => Ok(df.height() as f64),
    }
}

fn entropy_for_columns(
    df: &DataFrame,
    columns: &[String],
    weight_column: Option<&str>,
) -> PolarsResult<f64> {
    if columns.is_empty() {
        return Ok(0.0);
    }

    let weight_name = "__orthogonality_weight__";
    let group_exprs = columns.iter().map(col).collect::<Vec<Expr>>();
    let lazy = df.clone().lazy().group_by(group_exprs);

    let grouped = match weight_column {
        Some(column) => lazy.agg([col(column).sum().alias(weight_name)]).collect()?,
        None => lazy.agg([len().alias(weight_name)]).collect()?,
    };

    let weights = grouped
        .column(weight_name)?
        .cast(&DataType::Float64)?
        .f64()?
        .into_no_null_iter()
        .collect::<Vec<f64>>();

    let total = total_weight(df, weight_column)?;
    if total <= 0.0 {
        return Ok(0.0);
    }

    let entropy = weights
        .into_iter()
        .filter(|weight| *weight > 0.0)
        .map(|weight| {
            let probability = weight / total;
            -probability * probability.log2()
        })
        .sum();

    Ok(entropy)
}

fn mutual_information(
    cache: &mut EntropyCache,
    df: &DataFrame,
    x: &str,
    y: &str,
    weight_column: Option<&str>,
) -> PolarsResult<f64> {
    let x_cols = vec![x.to_string()];
    let y_cols = vec![y.to_string()];
    let xy_cols = vec![x.to_string(), y.to_string()];

    let h_x = cache.entropy(df, &x_cols, weight_column)?;
    let h_y = cache.entropy(df, &y_cols, weight_column)?;
    let h_xy = cache.entropy(df, &xy_cols, weight_column)?;

    Ok(h_x + h_y - h_xy)
}

fn joint_mutual_information(
    cache: &mut EntropyCache,
    df: &DataFrame,
    x1: &str,
    x2: &str,
    y: &str,
    weight_column: Option<&str>,
) -> PolarsResult<f64> {
    let x1x2_cols = vec![x1.to_string(), x2.to_string()];
    let y_cols = vec![y.to_string()];
    let x1x2y_cols = vec![x1.to_string(), x2.to_string(), y.to_string()];

    let h_x1x2 = cache.entropy(df, &x1x2_cols, weight_column)?;
    let h_y = cache.entropy(df, &y_cols, weight_column)?;
    let h_x1x2y = cache.entropy(df, &x1x2y_cols, weight_column)?;

    Ok(h_x1x2 + h_y - h_x1x2y)
}

fn orthogonality_report(
    cache: &mut EntropyCache,
    df: &DataFrame,
    x1: &str,
    x2: &str,
    y: &str,
    weight_column: Option<&str>,
) -> PolarsResult<PairReport> {
    let i_y_x1 = mutual_information(cache, df, x1, y, weight_column)?;
    let i_y_x2 = mutual_information(cache, df, x2, y, weight_column)?;
    let i_y_x1x2 = joint_mutual_information(cache, df, x1, x2, y, weight_column)?;
    let coinformation = i_y_x1 + i_y_x2 - i_y_x1x2;
    let kappa = if i_y_x1x2 > 1e-15 {
        Some(coinformation / i_y_x1x2)
    } else {
        None
    };

    Ok(PairReport {
        metric_a: x1.to_string(),
        metric_b: x2.to_string(),
        i_y_a: i_y_x1,
        i_y_b: i_y_x2,
        i_y_ab: i_y_x1x2,
        coinformation,
        kappa,
    })
}

fn selected_metrics(target: &str, count_column: &str, raw: &[String]) -> PolarsResult<Vec<String>> {
    let available: std::collections::HashSet<String> = metric_columns(MetricType::All)
        .into_iter()
        .map(|name| name.to_string())
        .collect();

    if !raw.is_empty() {
        let mut cols = Vec::new();
        for metric in raw {
            if metric == target || metric == count_column {
                continue;
            }

            if !available.contains(metric) {
                return Err(PolarsError::ComputeError(
                    format!(
                        "Unknown metric column '{}'. Available columns: {}",
                        metric,
                        metric_columns(MetricType::All).join(", ")
                    )
                    .into(),
                ));
            }

            if !cols.iter().any(|existing| existing == metric) {
                cols.push(metric.clone());
            }
        }

        return Ok(cols);
    }

    Ok(metric_columns(MetricType::All)
        .into_iter()
        .map(|name| name.to_string())
        .filter(|name| name != target && name != count_column)
        .collect())
}

fn format_value(value: f64, precision: usize) -> String {
    format!("{value:.precision$}")
}

fn pair_lookup<'a>(reports: &'a [PairReport], a: &str, b: &str) -> Option<&'a PairReport> {
    reports.iter().find(|report| {
        (report.metric_a == a && report.metric_b == b)
            || (report.metric_a == b && report.metric_b == a)
    })
}

fn display_label(label: &str, detail: Option<&str>) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    let mut row = vec![
        Cell::new(label)
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
    ];
    if let Some(detail) = detail {
        row.push(Cell::new(detail).fg(Color::Grey));
    }

    table.set_header(row);
    println!("\n{table}");
}

fn render_scalar_table(title: &str, headers: &[String], values: &[f64], precision: usize) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("Metric").add_attribute(Attribute::Bold),
        Cell::new("I(Y; Xi) [bits]").add_attribute(Attribute::Bold),
    ]);

    for (header, value) in headers.iter().zip(values.iter()) {
        let color = if *value > 0.0 {
            Color::Green
        } else if *value < 0.0 {
            Color::Red
        } else {
            Color::Grey
        };

        table.add_row(vec![
            Cell::new(header).fg(Color::Cyan),
            Cell::new(format_value(*value, precision)).fg(color),
        ]);
    }

    display_label(title, None);
    println!("{table}");
}

fn render_pair_table(
    title: &str,
    metrics: &[String],
    getter: impl Fn(&str, &str) -> Option<(String, Option<Color>)>,
) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    let mut header = vec![Cell::new("Metric").add_attribute(Attribute::Bold)];
    for metric in metrics {
        header.push(
            Cell::new(metric)
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
        );
    }
    table.set_header(header);

    for metric_a in metrics {
        let mut row = vec![Cell::new(metric_a).fg(Color::Cyan)];
        for metric_b in metrics {
            let cell = if metric_a == metric_b {
                Cell::new("-").fg(Color::Grey)
            } else if let Some((value, color)) = getter(metric_a, metric_b) {
                let mut cell = Cell::new(value);
                if let Some(color) = color {
                    cell = cell.fg(color);
                }
                cell
            } else {
                Cell::new("N/A").fg(Color::Grey)
            };
            row.push(cell);
        }
        table.add_row(row);
    }

    display_label(title, None);
    println!("{table}");
}

fn main() -> PolarsResult<()> {
    let args = Args::parse();

    let file = File::open(&args.input)?;
    let df = ParquetReader::new(file).finish()?;

    let (prepared_df, target_column, target_label) = if args.binary {
        let prepared = df
            .lazy()
            .with_column(
                col(&args.target)
                    .lt_eq(lit(args.threshold))
                    .alias("target_binary"),
            )
            .collect()?;
        (
            prepared,
            String::from("target_binary"),
            format!("{} <= {}", args.target, args.threshold),
        )
    } else {
        (df, args.target.clone(), args.target.clone())
    };

    if !prepared_df
        .get_column_names()
        .iter()
        .any(|name| name.as_str() == args.count_column)
    {
        return Err(PolarsError::ComputeError(
            format!(
                "Count column '{}' not found in input parquet",
                args.count_column
            )
            .into(),
        ));
    }

    let weight_column = Some(args.count_column.as_str());

    let metrics = selected_metrics(&target_column, &args.count_column, &args.metrics)?;
    ensure_columns_exist(&prepared_df, &metrics)?;
    ensure_columns_exist(&prepared_df, &[target_column.clone()])?;

    if metrics.len() < 2 {
        return Err(PolarsError::ComputeError(
            "Need at least two metric columns to compute orthogonality".into(),
        ));
    }

    let total_pairs = metrics.len() * (metrics.len() - 1) / 2;
    let pb = ProgressBar::new(total_pairs as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) | {msg}",
        )
        .unwrap()
        .progress_chars("█▉▊▋▌▍▎▏  "),
    );

    let mut cache = EntropyCache::new();
    let mut reports = Vec::with_capacity(total_pairs);

    for i in 0..metrics.len() {
        for j in (i + 1)..metrics.len() {
            let metric_a = &metrics[i];
            let metric_b = &metrics[j];
            pb.set_message(format!("{metric_a} vs {metric_b}"));

            let report = orthogonality_report(
                &mut cache,
                &prepared_df,
                metric_a,
                metric_b,
                &target_column,
                weight_column,
            )?;

            reports.push(report);
            pb.inc(1);
        }
    }

    pb.finish_with_message("Orthogonality analysis complete");

    println!("\n====================== Orthogonality Report ======================");
    println!("Input file                 : {}", args.input);
    println!("Target column              : {}", args.target);
    println!("Target label                : {}", target_label);
    println!("Target used in analysis     : {}", target_column);
    println!(
        "Weight column               : {}",
        weight_column.unwrap_or("(row count)")
    );
    println!("Metric columns             : {}", metrics.join(", "));
    println!("Total pair reports          : {}", reports.len());
    println!("=================================================================\n");

    let mut sorted_metrics = metrics.clone();
    sorted_metrics.sort();

    let info_headers: Vec<String> = sorted_metrics.clone();
    let info_values: Vec<f64> = sorted_metrics
        .iter()
        .map(|metric| {
            reports
                .iter()
                .find(|report| &report.metric_a == metric || &report.metric_b == metric)
                .map(|report| {
                    if &report.metric_a == metric {
                        report.i_y_a
                    } else {
                        report.i_y_b
                    }
                })
                .unwrap_or(0.0)
        })
        .collect();

    // Export CSV outputs if specified
    if let Some(csv_path) = &args.output_csv {
        let mut file = std::fs::File::create(csv_path).map_err(|e| {
            PolarsError::ComputeError(format!("Failed to create CSV: {}", e).into())
        })?;

        writeln!(
            file,
            "metric_a,metric_b,I_Y_a,I_Y_b,I_Y_ab,coinformation,kappa,relationship"
        )
        .map_err(|e| PolarsError::ComputeError(format!("Failed to write CSV: {}", e).into()))?;

        for report in &reports {
            let relationship = if report.coinformation > 1e-10 {
                "REDUNDANCY"
            } else if report.coinformation < -1e-10 {
                "SYNERGY"
            } else {
                "ORTHOGONAL"
            };

            let kappa_str = report
                .kappa
                .map(|v| format!("{}", v))
                .unwrap_or_else(|| "NA".to_string());

            writeln!(
                file,
                "{},{},{:.6},{:.6},{:.6},{:.6},{},{}",
                report.metric_a,
                report.metric_b,
                report.i_y_a,
                report.i_y_b,
                report.i_y_ab,
                report.coinformation,
                kappa_str,
                relationship
            )
            .map_err(|e| PolarsError::ComputeError(format!("Failed to write CSV: {}", e).into()))?;
        }

        eprintln!("Pair reports CSV written to: {}", csv_path);
    }

    if let Some(mi_path) = &args.output_mi_csv {
        let mut file = std::fs::File::create(mi_path).map_err(|e| {
            PolarsError::ComputeError(format!("Failed to create CSV: {}", e).into())
        })?;

        writeln!(file, "metric,I_Y_Xi")
            .map_err(|e| PolarsError::ComputeError(format!("Failed to write CSV: {}", e).into()))?;

        for (header, value) in info_headers.iter().zip(info_values.iter()) {
            writeln!(file, "{},{:.6}", header, value).map_err(|e| {
                PolarsError::ComputeError(format!("Failed to write CSV: {}", e).into())
            })?;
        }

        eprintln!("Mutual information CSV written to: {}", mi_path);
    }

    // Print display output only if no CSV outputs were specified
    if args.output_csv.is_none() && args.output_mi_csv.is_none() {
        render_scalar_table(
            "Mutual informations I(Y; Xi)",
            &info_headers,
            &info_values,
            6,
        );

        render_pair_table(
            "Coinformations table (interaction information)",
            &metrics,
            |a, b| {
                pair_lookup(&reports, a, b).map(|report| {
                    let color = if report.coinformation > 1e-10 {
                        Some(Color::Green)
                    } else if report.coinformation < -1e-10 {
                        Some(Color::Red)
                    } else {
                        Some(Color::Grey)
                    };

                    (format!("{:+.4}", report.coinformation), color)
                })
            },
        );

        render_pair_table(
            "Joint mutual information table I(Y; Xi, Xj)",
            &metrics,
            |a, b| {
                pair_lookup(&reports, a, b)
                    .map(|report| (format!("{:.4}", report.i_y_ab), Some(Color::Yellow)))
            },
        );

        render_pair_table(
            "Kappa table (normalized coinformation)",
            &metrics,
            |a, b| {
                pair_lookup(&reports, a, b).and_then(|report| {
                    report.kappa.map(|value| {
                        let color = if value > 0.05 {
                            Some(Color::Green)
                        } else if value < -0.05 {
                            Some(Color::Red)
                        } else {
                            Some(Color::Grey)
                        };

                        (format!("{value:+.4}"), color)
                    })
                })
            },
        );

        println!("\nPairwise reports");
        println!("---------------");
        for report in &reports {
            let label = if report.coinformation > 1e-10 {
                "REDUNDANCY"
            } else if report.coinformation < -1e-10 {
                "SYNERGY"
            } else {
                "ORTHOGONAL"
            };

            println!(
                "{} vs {} | I(Y;X1)={:.6} | I(Y;X2)={:.6} | I(Y;X1,X2)={:.6} | C={:+.6} | κ={} | {}",
                report.metric_a,
                report.metric_b,
                report.i_y_a,
                report.i_y_b,
                report.i_y_ab,
                report.coinformation,
                report
                    .kappa
                    .map(|value| format!("{value:+.6}"))
                    .unwrap_or_else(|| String::from("N/A")),
                label,
            );
        }
    }

    Ok(())
}
