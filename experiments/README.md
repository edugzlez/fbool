# Machine Learning Experiments

This folder contains the experiment runner and dataset used for model training.

Current locations:

- Main CLI: `experiments/cli.py`
- Train command module: `experiments/commands/train.py`
- Backward-compatible train entrypoint: `experiments/train.py`
- Dataset parquet: `experiments/results/dataset.parquet`
- Output CSVs: `experiments/results/`

The training CLI uses Typer subcommands:

- `compare`
- `mlp`
- `decision-tree`
- `random-forest`
- `extra-trees`
- `gradient-boosting`
- `logistic-regression`
- `linear-svm`
- `knn`
- `naive-bayes`
- `lightgbm`
- `pytorch-mlp` (optional, requires `torch`)
- `pytorch-mlp-smc2026` (paper-aligned preset, requires `torch`)

## 1. Quick Start

From repo root:

```bash
uv run experiments/cli.py --help
```

Training subcommands:

```bash
uv run experiments/cli.py train --help
```

From inside `experiments/`:

```bash
cd experiments
uv run cli.py --help
uv run cli.py train --help
```

If you need LightGBM on demand:

```bash
uv run --with lightgbm experiments/cli.py train lightgbm --help

# PyTorch command (optional dependency)
uv run --with torch experiments/cli.py train pytorch-mlp --help
uv run --with torch experiments/cli.py train pytorch-mlp-smc2026 --help
```

## 2. Dataset

Default dataset path to use in commands:

- `experiments/results/dataset.parquet`

Expected minimum columns:

- `count`
- `min_gates` (default target)
- numeric feature columns (`entanglement`, `entropy`, `spectral_entropy`, etc.)

Notes:

- `npn_repr` is automatically excluded from features.
- Training/evaluation uses `count` as sample weight where applicable.

## 3. Typical Commands

Compare several models (multiclass):

```bash
uv run experiments/cli.py train \
  --input experiments/results/dataset.parquet \
  --target min_gates \
  --count-col count \
  --test-size 0.2 \
  --seed 42 \
  --output-csv experiments/results/multiclass_results.csv \
  compare --models mlp,random_forest,lightgbm
```

Compare several models (binary):

```bash
uv run experiments/cli.py train \
  --input experiments/results/dataset.parquet \
  --target min_gates \
  --count-col count \
  --binary \
  --threshold 9 \
  --test-size 0.2 \
  --seed 42 \
  --output-csv experiments/results/binary_results_t9.csv \
  compare --models all
```

Train a single model with custom hyperparameters:

```bash
uv run experiments/cli.py train \
  --input experiments/results/dataset.parquet \
  mlp --hidden-layers 64,32 --max-iter 500 --alpha 0.0005

uv run experiments/cli.py train \
  --input experiments/results/dataset.parquet \
  random-forest --n-estimators 500 --max-depth 18 --min-samples-leaf 2

uv run experiments/cli.py train \
  --input experiments/results/dataset.parquet \
  lightgbm --n-estimators 700 --learning-rate 0.03 --num-leaves 63

uv run --with torch experiments/cli.py train \
  --input experiments/results/dataset.parquet \
  --normalize \
  --seed 42 \
  pytorch-mlp --hidden-layers 128,64 --epochs 150 --batch-size 256 --learning-rate 0.001 --optimizer adam --deterministic

# SMC2026-aligned MLP preset (GELU + BatchNorm + AdamW + OneCycle)
uv run --with torch experiments/cli.py train \
  --input experiments/results/dataset.parquet \
  --metrics entropy_based_entanglement,entanglement,total_influence,spectral_entropy,repetitiveness,nolinearity,entropy,sensitivity,certificate_complexity,counting,degree \
  --binary --threshold 9 \
  --max-rows 2000000 \
  --test-size 0.1 \
  --seed 42 \
  --output-csv experiments/results/smc2026_pytorch_mlp.csv \
  pytorch-mlp-smc2026 --device auto --deterministic
```

## 4. Verbose Training

Pipeline-level logs:

```bash
uv run experiments/cli.py train \
  --input experiments/results/dataset.parquet \
  --verbose \
  compare --models random_forest
```

Internal training logs (epochs/iterations where supported):

```bash
uv run experiments/cli.py train \
  --input experiments/results/dataset.parquet \
  --train-verbose 1 \
  mlp --max-iter 100

uv run experiments/cli.py train \
  --input experiments/results/dataset.parquet \
  --train-verbose 1 \
  logistic-regression

uv run experiments/cli.py train \
  --input experiments/results/dataset.parquet \
  --train-verbose 1 \
  --lgb-log-period 25 \
  lightgbm

uv run --with torch experiments/cli.py train \
  --input experiments/results/dataset.parquet \
  --train-verbose 1 \
  pytorch-mlp --epochs 20
```

## 5. Weighted Metrics

- Training uses `sample_weight=count` when model `fit` supports it.
- Final reported metrics are always weighted on test data with `count`.
- Metrics include `acc_w`, `f1_macro_w`, `precision_macro_w`, `recall_macro_w`, `bal_acc_w`, and `f1_binary_w` in binary mode.

## 6. Reproducible PyTorch MLP Runs

To make runs easy to reproduce and contrast across machines/users:

- Keep `--seed` fixed.
- Use `--normalize` for feature scaling consistency.
- Use `pytorch-mlp --deterministic` to force deterministic behavior where supported.
- Save outputs with `--output-csv` so all metrics are archived.

If you need to match the paper protocol:

- Use `--binary --threshold 9`.
- Use `--max-rows 2000000 --test-size 0.1` (stratified sample then 90/10 split).
- Use `pytorch-mlp-smc2026`.

Example reproducible run:

```bash
uv run --with torch experiments/cli.py train \
  --input experiments/results/dataset.parquet \
  --metrics sensitivity,influence,fp_k3,fp_k4,spectral_entropy \
  --binary --threshold 9 \
  --normalize \
  --seed 42 \
  --output-csv experiments/results/pytorch_mlp_binary_t9.csv \
  pytorch-mlp \
    --hidden-layers 128,64 \
    --epochs 150 \
    --batch-size 256 \
    --learning-rate 0.001 \
    --weight-decay 0.0 \
    --optimizer adam \
    --device auto \
    --deterministic
```

## 7. Cargo-backed Subcommands

The main CLI also wraps common Cargo workflows:

```bash
# Build full workspace in release mode
uv run experiments/cli.py build

# Run all tests
uv run experiments/cli.py test

# Run clippy
uv run experiments/cli.py clippy

# Forward args to fbool-cli binary
uv run experiments/cli.py fbool-cli entanglement majority -n 4

# Run a bin from fbool-experiments
uv run experiments/cli.py experiments-bin compute-metrics -- --help
```

## 8. Complete Pipeline for Paper Replication

This section describes the complete pipeline for generating the dataset, computing metrics, and producing analysis table referenced in the SMC2026 paper.

### Step 1: Generate NPN Representative Table

Create the canonical (NPN-equivalent) representatives for all 4+ billion 5-variable Boolean functions:

```bash
cd experiments
uv run cli.py experiments-bin npn-create --output-path results/npn_table.parquet
```

Alternatively, specify custom parameters:

```bash
uv run cli.py experiments-bin npn-create \
  --output-path results/npn_full.parquet \
  --max-funs 4294967296  # 2^32 functions
```

**Output**: Parquet file with columns:
- `npn_repr` (u32): NPN representative value
- `count` (u64): How many 5-variable functions reduce to this representative

### Step 2: Compute Boolean Function Metrics

Add metric columns (entanglement, gates, entropy, etc.) to the NPN table:

```bash
uv run cli.py experiments-bin compute-metrics \
  -i results/npn_table.parquet \
  -o results/dataset.parquet \
  -m all
```

To compute only specific metrics:

```bash
uv run cli.py experiments-bin compute-metrics \
  -i results/npn_table.parquet \
  -o results/dataset.parquet \
  -m Gates,Entanglement,SpectralEntropy,Nolinearity,Sensitivity,CertificateComplexity
```

**Available metrics (case-insensitive with dashes):**
- `gates` → min_gates (minimum AND/OR/NOT gates required)
- `entanglement`
- `entanglement-entropy` → entanglement_entropy
- `influence`
- `spectral-entropy` → spectral_entropy
- `nolinearity` → nolinearity
- `simple-entropy` → entropy
- `sensitivity` → sensitivity
- `certificate-complexity` → certificate_complexity
- `degree` → degree
- `counting` → counting
- `repetitiveness` → repetitiveness
- `fragmentation-profile` → fp_k0, fp_k1, ..., fp_k5 (6 columns)
- `fragmentation-peak` → k_star, s_max
- `all` → compute all metrics (default)

**Output**: Parquet file with original columns + metric columns

### Step 3: Analyze Bayes Optimal Upper Bound

Compute the theoretical maximum classification accuracy using the best possible classifier with given features:

```bash
# Multiclass prediction (all gate counts)
uv run cli.py experiments-bin bayes-optimal \
  --input results/dataset.parquet \
  --target min_gates \
  --metrics entanglement,entropy,spectral_entropy

# Binary classification (min_gates <= 9)
uv run cli.py experiments-bin bayes-optimal \
  --input results/dataset.parquet \
  --target min_gates \
  --threshold 9 \
  --binary \
  --metrics entanglement,entropy,spectral_entropy,sensitivity
```

Export results to CSV:

```bash
uv run cli.py experiments-bin bayes-optimal \
  --input results/dataset.parquet \
  --target min_gates \
  --threshold 9 \
  --binary \
  --metrics entanglement,entropy,spectral_entropy,sensitivity,nolinearity \
  --output-csv results/bayes_optimal_binary_t9.csv
```

**Output CSV columns:**
- `input_file`: Path to input parquet
- `features`: Comma-separated list of feature metrics
- `mode`: "binary" or "multiclass"
- `target_rule`: Rule applied (e.g., "min_gates <= 9")
- `count_column`: Weight column name
- `correct_mass`: Number of correctly classified functions (majority voting)
- `max_accuracy_percent`: Maximum achievable accuracy (%)

### Step 4: Analyze Metric Orthogonality

Compute mutual information, coinformation (interaction information), and orthogonality coefficients (κ) between metric pairs:

```bash
# Compute all pairwise orthogonality relationships
uv run cli.py experiments-bin orthogonality \
  --input results/dataset.parquet \
  --target min_gates \
  --metrics entanglement,entropy,spectral_entropy,sensitivity,nolinearity

# Binary classification variant
uv run cli.py experiments-bin orthogonality \
  --input results/dataset.parquet \
  --target min_gates \
  --threshold 9 \
  --binary \
  --metrics entanglement,entropy,spectral_entropy,sensitivity,nolinearity,degree
```

Export pair relationships to CSV:

```bash
uv run cli.py experiments-bin orthogonality \
  --input results/dataset.parquet \
  --target min_gates \
  --threshold 9 \
  --binary \
  --metrics entanglement,entropy,spectral_entropy,sensitivity,nolinearity,degree \
  --output-csv results/orthogonality_pairs.csv \
  --output-mi-csv results/mutual_information.csv
```

**Output CSV files:**

- `orthogonality_pairs.csv`: Pairwise analysis
  - `metric_a`, `metric_b`: Metric names
  - `I_Y_a`, `I_Y_b`: Individual mutual informations (bits)
  - `I_Y_ab`: Joint mutual information (bits)
  - `coinformation`: I(Y;Xi) + I(Y;Xj) - I(Y;Xi,Xj) (interaction)
  - `kappa`: Normalized coinformation (κ = C / I(Y;Xi,Xj))
  - `relationship`: "REDUNDANCY", "SYNERGY", or "ORTHOGONAL"

- `mutual_information.csv`: Summary of mutual informations
  - `metric`: Metric name
  - `I_Y_Xi`: I(Y; Xi) in bits

### Step 5: Complete Workflow (Full Dataset)

To replicate the exact paper protocol, use the provided automation scripts:

#### Option A: Automated Full Pipeline

Run the complete workflow (NPN generation → metrics → analysis) with a single command:

```bash
cd experiments
bash scripts/full_pipeline.sh                    # Default: ./_results
bash scripts/full_pipeline.sh custom_output/    # Custom output directory
```

This will:
1. Generate NPN table
2. Compute all metrics 
3. Run Bayes optimal analysis (binary + multiclass)
4. Run orthogonality analysis
5. Export all results to CSV

**Expected Duration:** 4-8 hours depending on hardware (GPU/parallel computation)

#### Option A.1: Quick Smoke Test (Validation)

Before running the full pipeline, validate the workflow with a subset:

```bash
cd experiments
bash scripts/smoke_test.sh                # Default: ./_smoke_test
bash scripts/smoke_test.sh test_output/   # Custom output directory
```

This runs the same pipeline but with only ~100K functions instead of 4B, completing in ~1-2 minutes. Use this to:
- Validate the workflow and dependencies
- Verify CSV output formats
- Test on smaller datasets for parameter exploration

**Expected Duration:** 1-2 minutes

**Output files:**
```
_results/
├── npn_table.parquet                        # All NPN representatives (with counts)
├── dataset.parquet                 # Metrics for NPN functions
├── bayes_optimal_binary_t9.csv              # Theoretical max accuracy (binary)
├── bayes_optimal_multiclass.csv             # Theoretical max accuracy (multiclass)
├── orthogonality_binary_t9.csv              # Pairwise metric relationships
├── mutual_info_binary_t9.csv                # Summary of metric information content
├── orthogonality_multiclass.csv             # Pairwise relationships (multiclass)
└── mutual_info_multiclass.csv               # Summary (multiclass)
```

#### Option B: Quick Analysis (Reuse Existing Metrics)

If you already have a metrics parquet file and want to run different analyses:

```bash
cd experiments
bash scripts/quick_analysis.sh _results/dataset.parquet
```

This runs:
- Binary Bayes optimal (threshold 9 and 5)
- Multiclass analysis
- Orthogonality and mutual information (binary, t=9)
- Subset analysis (4 core metrics)

Much faster than full pipeline since it skips metric computation.

#### Option C: Manual Workflow (Full Control)

Run each step individually:

```bash
cd experiments

# Step 1: Generate NPN table
uv run cli.py experiments-bin npn-create --output-path results/npn_table.parquet

# Step 2: Compute all metrics  
uv run cli.py experiments-bin compute-metrics \
  -i results/npn_table.parquet \
  -o results/dataset.parquet \
  -m All

# Step 3a: Binary Bayes optimal (threshold=9)
uv run cli.py experiments-bin bayes-optimal \
  --input results/dataset.parquet \
  --target min_gates \
  --binary \
  --threshold 9 \
  --metrics entanglement,entanglement_entropy,influence,spectral_entropy,repetitiveness,nolinearity,entropy,sensitivity,certificate_complexity,counting,degree \
  --output-csv results/bayes_optimal_t9.csv

# Step 3b: Multiclass Bayes optimal
uv run cli.py experiments-bin bayes-optimal \
  --input results/dataset.parquet \
  --target min_gates \
  --metrics entanglement,entanglement_entropy,influence,spectral_entropy,repetitiveness,nolinearity,entropy,sensitivity,certificate_complexity,counting,degree \
  --output-csv results/bayes_optimal_multiclass.csv

# Step 4a: Orthogonality (binary)
uv run cli.py experiments-bin orthogonality \
  --input results/dataset.parquet \
  --target min_gates \
  --binary \
  --threshold 9 \
  --metrics entanglement,entanglement_entropy,influence,spectral_entropy,repetitiveness,nolinearity,entropy,sensitivity,certificate_complexity,counting,degree \
  --output-csv results/orthogonality_t9.csv \
  --output-mi-csv results/mutual_info_t9.csv

# Step 4b: Orthogonality (multiclass)
uv run cli.py experiments-bin orthogonality \
  --input results/dataset.parquet \
  --target min_gates \
  --metrics entanglement,entanglement_entropy,influence,spectral_entropy,repetitiveness,nolinearity,entropy,sensitivity,certificate_complexity,counting,degree \
  --output-csv results/orthogonality_multiclass.csv \
  --output-mi-csv results/mutual_info_multiclass.csv
```

### Output Files

After running the complete pipeline, you'll have:

```
experiments/results/
├── npn_table.parquet                   # NPN representatives (with counts)
├── dataset.parquet            # All NPN functions with metrics
├── bayes_optimal_t9.csv                # Binary (t=9) theoretical accuracy
├── bayes_optimal_multiclass.csv        # Multiclass theoretical accuracy
├── orthogonality_t9.csv                # Pairwise metric relationships (binary, t=9)
├── mutual_info_t9.csv                  # Mutual information summary (binary, t=9)
├── orthogonality_multiclass.csv        # Pairwise metric relationships (multiclass)
└── mutual_info_multiclass.csv          # Mutual information summary (multiclass)
```

**CSV File Descriptions:**

- **bayes_optimal_*.csv**: One row per analysis
  - `input_file`: Path to input metrics parquet
  - `features`: Metrics used for classification
  - `mode`: "binary" or "multiclass"
  - `target_rule`: Rule applied (e.g., "min_gates <= 9")
  - `correct_mass`: Functions correctly classified by majority vote
  - `max_accuracy_percent`: Theoretical maximum accuracy achievable

- **orthogonality_*.csv**: One row per metric pair
  - `metric_a`, `metric_b`: Pair of metrics
  - `I_Y_a`, `I_Y_b`: Mutual information with target [bits]
  - `I_Y_ab`: Joint mutual information [bits]
  - `coinformation`: Interaction information (redundancy/synergy indicator)
  - `kappa`: Normalized coinformation (κ)
  - `relationship`: "REDUNDANCY", "SYNERGY", or "ORTHOGONAL"

- **mutual_info_*.csv**: Summary of metric information
  - `metric`: Metric name
  - `I_Y_Xi`: Mutual information with target [bits]

### Key Parameters for Paper Alignment

From "Approaching Circuit Complexity with Explainable Metrics (SMC2026)":

| Parameter | Value | Notes |
|-----------|-------|-------|
| Dataset | All 2^32 5-variable functions | Partitioned by min_gates |
| Threshold | 9 gates | Binary classification: simple (≤9) vs complex (>9) |
| Metrics | 11 computed | entanglement, entropy, spectral_entropy, sensitivity, certifticate_complexity, and 6 others |
| Train/Test Split | 90/10 stratified | 1.8M functions training, 200K testing |
| Target | min_gates (binary) | min_gates ≤ 9 is positive class |
| ML Models | Random Forest, MLP (PyTorch) | Baseline classical ML + modern deep learning |
| MLP Architecture | [256,512,256,128] | Hidden layers with GELU, BatchNorm, dropout (0.3,0.3,0.2) |
| Optimizer | AdamW | lr=1e-3, weight_decay=1e-4 |
| Scheduler | OneCycleLR | max_lr=1e-2, 15 epochs, batch_size=4096 |

### Running the Paper-Aligned MLP

After generating the dataset (Steps 1-2), train the exact MLP from the paper:

```bash
# Full dataset, 90/10 split, 15 epochs, batch 4096
uv run --with torch cli.py train \
  --input results/dataset.parquet \
  --metrics entanglement,entanglement_entropy,influence,spectral_entropy,repetitiveness,nolinearity,entropy,sensitivity,certificate_complexity,counting,degree \
  --binary \
  --threshold 9 \
  --max-rows 2000000 \
  --test-size 0.1 \
  --normalize \
  --seed 42 \
  --output-csv results/pytorch_mlp_smc2026_full.csv \
  pytorch-mlp-smc2026 \
    --device cuda \
    --deterministic
```

Expected results (from paper): ~76.42% test accuracy with all 11 metrics

