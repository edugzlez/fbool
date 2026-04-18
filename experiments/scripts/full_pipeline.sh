#!/bin/bash
# Full pipeline for Boolean function dataset analysis
# Replicates the experimental setup from SMC2026 paper
# Usage: bash full_pipeline.sh [output_dir]

set -e

OUTPUT_DIR="${1:-./_results}"
# Convert to absolute path
OUTPUT_DIR="$(cd "$(dirname "$OUTPUT_DIR")" 2>/dev/null && pwd)/$(basename "$OUTPUT_DIR")"
mkdir -p "$OUTPUT_DIR"

echo "=========================================="
echo "Boolean Function Analysis Pipeline"
echo "=========================================="
echo "Output directory: $OUTPUT_DIR"
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

cd "$(dirname "$0")/.."

# Step 1: Generate NPN table
echo -e "${BLUE}[STEP 1]${NC} Generating NPN representative table..."
uv run cli.py experiments-bin npn-create \
  --output-path "$OUTPUT_DIR/npn_table.parquet"
echo -e "${GREEN}✓ NPN table created${NC}"
echo ""

# Step 2: Compute all metrics
echo -e "${BLUE}[STEP 2]${NC} Computing Boolean function metrics..."
uv run cli.py experiments-bin compute-metrics \
  -i "$OUTPUT_DIR/npn_table.parquet" \
  -o "$OUTPUT_DIR/npn_with_metrics.parquet" \
  -m all
echo -e "${GREEN}✓ Metrics computed${NC}"
echo ""

# Step 3: Bayes optimal analysis
echo -e "${BLUE}[STEP 3]${NC} Computing Bayes optimal upper bounds..."
uv run cli.py experiments-bin bayes-optimal \
  --input "$OUTPUT_DIR/npn_with_metrics.parquet" \
  --target min_gates \
  --binary \
  --threshold 9 \
  --metrics entanglement,entanglement_entropy,influence,spectral_entropy,repetitiveness,nolinearity,entropy,sensitivity,certificate_complexity,counting,degree \
  --output-csv "$OUTPUT_DIR/bayes_optimal_binary_t9.csv"
echo -e "${GREEN}✓ Bayes optimal analysis complete${NC}"
cat "$OUTPUT_DIR/bayes_optimal_binary_t9.csv"
echo ""

# Step 4: Multiclass Bayes optimal
echo -e "${BLUE}[STEP 4]${NC} Computing Bayes optimal (multiclass - all gate counts)..."
uv run cli.py experiments-bin bayes-optimal \
  --input "$OUTPUT_DIR/npn_with_metrics.parquet" \
  --target min_gates \
  --metrics entanglement,entanglement_entropy,influence,spectral_entropy,repetitiveness,nolinearity,entropy,sensitivity,certificate_complexity,counting,degree \
  --output-csv "$OUTPUT_DIR/bayes_optimal_multiclass.csv"
echo -e "${GREEN}✓ Multiclass analysis complete${NC}"
echo ""

# Step 5: Orthogonality analysis (binary)
echo -e "${BLUE}[STEP 5]${NC} Analyzing metric orthogonality (binary classification)..."
uv run cli.py experiments-bin orthogonality \
  --input "$OUTPUT_DIR/npn_with_metrics.parquet" \
  --target min_gates \
  --binary \
  --threshold 9 \
  --metrics entanglement,entanglement_entropy,influence,spectral_entropy,repetitiveness,nolinearity,entropy,sensitivity,certificate_complexity,counting,degree \
  --output-csv "$OUTPUT_DIR/orthogonality_binary_t9.csv" \
  --output-mi-csv "$OUTPUT_DIR/mutual_info_binary_t9.csv"
echo -e "${GREEN}✓ Orthogonality (binary) analysis complete${NC}"
echo ""

# Step 6: Orthogonality analysis (multiclass)
echo -e "${BLUE}[STEP 6]${NC} Analyzing metric orthogonality (multiclass)..."
uv run cli.py experiments-bin orthogonality \
  --input "$OUTPUT_DIR/npn_with_metrics.parquet" \
  --target min_gates \
  --metrics entanglement,entanglement_entropy,influence,spectral_entropy,repetitiveness,nolinearity,entropy,sensitivity,certificate_complexity,counting,degree \
  --output-csv "$OUTPUT_DIR/orthogonality_multiclass.csv" \
  --output-mi-csv "$OUTPUT_DIR/mutual_info_multiclass.csv"
echo -e "${GREEN}✓ Orthogonality (multiclass) analysis complete${NC}"
echo ""

echo "=========================================="
echo -e "${GREEN}Pipeline Complete!${NC}"
echo "=========================================="
echo ""
echo "Output files:"
ls -lh "$OUTPUT_DIR"/*.csv 2>/dev/null || echo "  (No CSV files generated)"
ls -lh "$OUTPUT_DIR"/*.parquet 2>/dev/null || echo "  (No Parquet files generated)"
echo ""
echo "Key results:"
echo "  • Bayes optimal (binary, t=9): $OUTPUT_DIR/bayes_optimal_binary_t9.csv"
echo "  • Bayes optimal (multiclass): $OUTPUT_DIR/bayes_optimal_multiclass.csv"
echo "  • Orthogonality (binary, t=9): $OUTPUT_DIR/orthogonality_binary_t9.csv"
echo "  • Mutual information (binary, t=9): $OUTPUT_DIR/mutual_info_binary_t9.csv"
echo "  • Orthogonality (multiclass): $OUTPUT_DIR/orthogonality_multiclass.csv"
echo "  • Mutual information (multiclass): $OUTPUT_DIR/mutual_info_multiclass.csv"
echo ""
