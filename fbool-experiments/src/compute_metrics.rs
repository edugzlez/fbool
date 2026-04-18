use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
#[path = "metrics.rs"]
mod metrics;

use metrics::{METRIC_ORDER, Metric, MetricType, metric_impl};
use polars::prelude::*;
use rayon::prelude::*;
use std::fs::File;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Cómputo de métricas sobre NPN representantes"
)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long, value_enum, default_value_t = MetricType::All)]
    metric: MetricType,
}

/*
    Main Program
*/
fn apply_metric(
    df: DataFrame,
    metric: Box<dyn Metric>,
    metric_name: &str,
) -> PolarsResult<DataFrame> {
    let npn_col = df.column("npn_repr")?.u32()?;
    let num_rows = npn_col.len();
    let npn_values: Vec<Option<u32>> = npn_col.into_iter().collect();

    let pb = ProgressBar::new(num_rows as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) | {per_sec} | ETA: {eta} | {msg}"
        )
        .unwrap()
        .progress_chars("█▉▊▋▌▍▎▏  ")
    );
    pb.set_message(metric_name.to_string());

    // 1. Envolvemos en Arc
    let metric_arc = Arc::new(metric);

    let names = metric_arc.column_names();
    let pb_arc = Arc::new(pb);

    // 2. Clonamos los punteros para el cierre de Rayon
    // Esto es lo que resuelve el error de "move out of metric"
    let metric_for_thread = Arc::clone(&metric_arc);
    let pb_for_thread = Arc::clone(&pb_arc);

    let results: Vec<Vec<AnyValue<'static>>> = npn_values
        .into_par_iter()
        .map(move |opt_v| {
            // <--- 'move' captura los clones de arriba
            let res = match opt_v {
                Some(v) => metric_for_thread.compute(v),
                None => (0..names.len()).map(|_| AnyValue::Null).collect(),
            };
            pb_for_thread.inc(1);
            res
        })
        .collect();

    pb_arc.finish_with_message("Cómputo finalizado");

    // 3. Reconstrucción de columnas (usamos metric_arc para los nombres)
    let mut new_columns = Vec::new();
    for (i, &name) in metric_arc.column_names().iter().enumerate() {
        let col_data: Vec<AnyValue> = results.iter().map(|row| row[i].clone()).collect();
        let s = Series::from_any_values(name.into(), &col_data, true)?;
        new_columns.push(Column::from(s));
    }

    df.hstack(&new_columns)
}

fn main() -> PolarsResult<()> {
    let args = Args::parse();

    // Leer
    let mut df = ParquetReader::new(File::open(&args.input)?).finish()?;

    // Seleccionar métrica basado en el input de CLI
    df = match args.metric {
        MetricType::All => {
            let mut out = df;
            for metric in METRIC_ORDER {
                let metric_impl = metric_impl(metric).unwrap();
                out = apply_metric(out, metric_impl, &format!("metric={metric:?}"))?;
            }
            out
        }
        metric => {
            let metric_impl = metric_impl(metric).unwrap();
            apply_metric(df, metric_impl, &format!("metric={metric:?}"))?
        }
    };

    df = df.lazy().sort(["npn_repr"], Default::default()).collect()?;

    // Guardar
    let file = File::create(&args.output)?;
    ParquetWriter::new(file).finish(&mut df)?;

    Ok(())
}
