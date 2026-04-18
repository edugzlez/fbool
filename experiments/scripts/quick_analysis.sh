#!/bin/sh
# Quick analysis pipeline for existing metrics parquet
# Use this after running full_pipeline.sh to re-analyze with different parameters
# Usage: sh scripts/quick_analysis.sh <metrics_parquet> [output_dir]

set -e

METRICS_FILE="${1:-_results/npn_with_metrics.parquet}"
OUTPUT_DIR="${2:-_results}"

resolve_path() {
  case "$1" in
    /*) printf "%s" "$1" ;;
    *) printf "%s/%s" "$PWD" "$1" ;;
  esac
}

METRICS_FILE="$(resolve_path "$METRICS_FILE")"
OUTPUT_DIR="$(resolve_path "$OUTPUT_DIR")"

if [ ! -f "$METRICS_FILE" ]; then
    echo "Error: Metrics file not found: $METRICS_FILE"
  echo "Usage: sh scripts/quick_analysis.sh <metrics_parquet> [output_dir]"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

echo "=========================================="
echo "Quick Analysis Pipeline"
echo "=========================================="
echo "Input metrics: $METRICS_FILE"
echo "Output directory: $OUTPUT_DIR"
echo ""

cd "$(dirname "$0")/.."

# Analysis 1: Binary classification (threshold 9)
echo "[1] Binary Bayes optimal (min_gates <= 9)..."
uv run cli.py experiments-bin bayes-optimal \
  --input "$METRICS_FILE" \
  --target min_gates \
  --binary \
  --threshold 9 \
  --metrics entanglement,entanglement_entropy,influence,spectral_entropy,repetitiveness,nolinearity,entropy,sensitivity,certificate_complexity,counting,degree \
  --output-csv "$OUTPUT_DIR/bayes_optimal_t9.csv"
echo "OK: $OUTPUT_DIR/bayes_optimal_t9.csv"

# Analysis 2: Binary classification (threshold 5)
echo "[2] Binary Bayes optimal (min_gates <= 5)..."
uv run cli.py experiments-bin bayes-optimal \
  --input "$METRICS_FILE" \
  --target min_gates \
  --binary \
  --threshold 5 \
  --metrics entanglement,entanglement_entropy,influence,spectral_entropy,repetitiveness,nolinearity,entropy,sensitivity,certificate_complexity,counting,degree \
  --output-csv "$OUTPUT_DIR/bayes_optimal_t5.csv"
echo "OK: $OUTPUT_DIR/bayes_optimal_t5.csv"

# Analysis 3: Multiclass
echo "[3] Multiclass Bayes optimal..."
uv run cli.py experiments-bin bayes-optimal \
  --input "$METRICS_FILE" \
  --target min_gates \
  --metrics entanglement,entanglement_entropy,influence,spectral_entropy,repetitiveness,nolinearity,entropy,sensitivity,certificate_complexity,counting,degree \
  --output-csv "$OUTPUT_DIR/bayes_optimal_multiclass.csv"
echo "OK: $OUTPUT_DIR/bayes_optimal_multiclass.csv"

# Analysis 4: Orthogonality binary
echo "[4] Orthogonality analysis (binary, t=9)..."
uv run cli.py experiments-bin orthogonality \
  --input "$METRICS_FILE" \
  --target min_gates \
  --binary \
  --threshold 9 \
  --metrics entanglement,entanglement_entropy,influence,spectral_entropy,repetitiveness,nolinearity,entropy,sensitivity,certificate_complexity,counting,degree \
  --output-csv "$OUTPUT_DIR/orthogonality_t9.csv" \
  --output-mi-csv "$OUTPUT_DIR/mi_t9.csv"
echo "OK: $OUTPUT_DIR/orthogonality_t9.csv"
echo "OK: $OUTPUT_DIR/mi_t9.csv"

# Analysis 5: Subset analysis (fewer metrics)
echo "[5] Subset analysis (core metrics only)..."
uv run cli.py experiments-bin bayes-optimal \
  --input "$METRICS_FILE" \
  --target min_gates \
  --binary \
  --threshold 9 \
  --metrics entanglement,entropy,spectral_entropy,sensitivity \
  --output-csv "$OUTPUT_DIR/bayes_optimal_core_metrics.csv"
echo "OK: $OUTPUT_DIR/bayes_optimal_core_metrics.csv"

echo ""
echo "=========================================="
echo "Analysis complete!"
echo "=========================================="
echo ""
echo "Results summary:"
echo "├── $OUTPUT_DIR/bayes_optimal_t9.csv"
echo "├── $OUTPUT_DIR/bayes_optimal_t5.csv"
echo "├── $OUTPUT_DIR/bayes_optimal_multiclass.csv"
echo "├── $OUTPUT_DIR/orthogonality_t9.csv"
echo "├── $OUTPUT_DIR/mi_t9.csv"
echo "└── $OUTPUT_DIR/bayes_optimal_core_metrics.csv"
echo ""
