# Analysis Scripts

Automated bash scripts for running the complete Boolean function analysis pipeline.

## Quick Start

### 1. Validate Installation (2 minutes)

```bash
bash smoke_test.sh _test
```

Tests the pipeline with ~100K functions to verify:
- Commands are correct
- Dependencies are installed
- CSV outputs are generated
- Paths are configured properly

Output: `./_test/bayes_optimal_subset_t9.csv`, `./_test/orthogonality_subset_t9.csv`, etc.

### 2. Run Full Pipeline (4-8 hours on typical hardware)

```bash
bash full_pipeline.sh _results
```

Generates analysis for all 4+ billion 5-variable Boolean functions:
- NPN table (~2GB parquet)
- Metrics (~5GB parquet)
- Bayes optimal analysis (binary + multiclass)
- Orthogonality analysis (binary + multiclass)

Output: `./_results/*.csv` (analysis files), `./_results/*.parquet` (datasets)

### 3. Re-run Analysis with Different Parameters (minutes)

If you already have metrics:

```bash
bash quick_analysis.sh ./_results/npn_with_metrics.parquet ./_results
```

Runs multiple analyses on the same metrics:
- Threshold 5 vs 9 vs multiclass
- Different metric subsets
- All saved to separate CSVs

## Script Details

### `smoke_test.sh`

**Purpose**: Validate workflow with small dataset

**Input**: None (generates subset automatically)

**Output**:
```
_test/
├── npn_subset.parquet              (~13K)
├── npn_subset_metrics.parquet       (~46K)
├── bayes_optimal_subset_t9.csv      (1 row)
├── orthogonality_subset_t9.csv      (6 rows for 4 metrics)
└── mi_subset_t9.csv                 (4 rows)
```

**Duration**: 1-2 minutes (CPU)

**When to use**:
- First time running the workflow
- Testing on a new machine
- Verifying dependencies
- Experimenting with command line options

### `full_pipeline.sh`

**Purpose**: Generate complete analysis dataset

**Input**: None (generates from scratch)

**Output**:
```
_results/
├── npn_table.parquet                    (~2GB, 4.2B functions)
├── npn_with_metrics.parquet             (~5GB, all metrics)
├── bayes_optimal_binary_t9.csv          (1 row)
├── bayes_optimal_multiclass.csv         (1 row)
├── orthogonality_binary_t9.csv          (55 rows for 11 metrics)
├── mutual_info_binary_t9.csv            (11 rows)
├── orthogonality_multiclass.csv         (55 rows)
└── mutual_info_multiclass.csv           (11 rows)
```

**Duration**: 4-8 hours depending on:
- CPU cores (parallel metric computation)
- Storage speed (parquet I/O)
- Available memory

**When to use**:
- Generating complete dataset for research
- Replicating paper results exactly
- Large-scale analysis

### `quick_analysis.sh`

**Purpose**: Re-analyze existing metrics with different parameters

**Input**: `<path_to_metrics.parquet>` from `full_pipeline.sh`

**Output**: Multiple CSV files with different threshold/metric combinations

**Example**:
```bash
bash quick_analysis.sh ./results/npn_with_metrics.parquet ./analysis_v2
```

Generates in `./analysis_v2/`:
```
bayes_optimal_t5.csv        # Threshold 5
bayes_optimal_t9.csv        # Threshold 9
bayes_optimal_multiclass.csv
orthogonality_t9.csv
mi_t9.csv
bayes_optimal_core_metrics.csv  # 4 metrics only
```

**Duration**: 5-20 minutes (much faster than full pipeline)

**When to use**:
- Parameter exploration (different thresholds)
- Feature subset evaluation
- Quick iterations during research

## Expected Directory Structure

```
experiments/
├── scripts/
│   ├── smoke_test.sh          ← Start here
│   ├── full_pipeline.sh       ← For complete analysis
│   ├── quick_analysis.sh      ← For parameter tuning
│   └── README.md              ← This file
├── cli.py                     ← Main CLI entry point
├── ANALYSIS_GUIDE.md          ← Interpret CSV outputs
└── README.md                  ← General documentation
```

## Troubleshooting

### "Command not found: uv"

Install UV: https://docs.astral.sh/uv/getting-started/

### "No such file or directory" in parquet output

The script creates directories automatically, but check:
- Parent directory exists and is writable
- Enough disk space (~10GB for full pipeline)
- Path doesn't have special characters

### Slow metric computation

Normal performance varies:
- Single-threaded: ~1000 metrics/second
- Multi-threaded: ~100K metrics/second on 8-core CPU

Expected times:
- ~100K functions: 1-5 seconds
- ~1M functions: 1-2 minutes
- ~2.5M functions: 3-5 minutes
- ~4.2B functions: 1-2 hours (parallel)

### Out of memory during metrics

The compute step uses parallel processing. If you run out of RAM:
- Close other applications
- Reduce `--max-funs` in `smoke_test.sh` temporarily
- Contact the paper authors if running on limited hardware

## Understanding the Workflow

```
✓ NPN table (4.2B unique representatives)
         ↓
✓ Compute metrics (entanglement, entropy, gates, etc.)
         ↓
╔═ Bayes Optimal ═╝  (theoretical accuracy bound)
║
╚═ Orthogonality ═   (metric relationships)

Result: CSV files ready for publication/ML pipeline
```

## Citation

If you use this pipeline, please cite:

```bibtex
@paper{smc2026,
  title={Approaching Circuit Complexity with Explainable Metrics},
  year={2026}
}
```

See [../README.md](../README.md) for full citation and methodology.
