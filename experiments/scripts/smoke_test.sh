#!/bin/bash
# Quick test run (subset of data) - useful for smoke testing and workflow validation
# Uses only ~100K functions instead of full 4B dataset
# Usage: bash smoke_test.sh [output_dir]

set -e

OUTPUT_DIR="${1:-./_smoke_test}"
# Convert to absolute path
OUTPUT_DIR="$(cd "$(dirname "$OUTPUT_DIR")" && pwd)/$(basename "$OUTPUT_DIR")"
mkdir -p "$OUTPUT_DIR"

echo "=========================================="
echo "Smoke Test (Subset Dataset)"
echo "=========================================="
echo "Output directory: $OUTPUT_DIR"
echo ""
echo "This will test the full pipeline with only ~100K functions"
echo "Useful for validating the workflow before running on full 4B dataset"
echo ""

GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

cd "$(dirname "$0")/.."

# Step 1: Generate small NPN table
echo -e "${BLUE}[STEP 1]${NC} Generating subset NPN table (~100K functions)..."
uv run cli.py experiments-bin npn-create \
  --output-path "$OUTPUT_DIR/npn_subset.parquet" \
  --max-funs 100000
echo -e "${GREEN}✓ NPN table created${NC}"
echo ""

# Step 2: Compute metrics
echo -e "${BLUE}[STEP 2]${NC} Computing metrics (subset)..."
uv run cli.py experiments-bin compute-metrics \
  -i "$OUTPUT_DIR/npn_subset.parquet" \
  -o "$OUTPUT_DIR/npn_subset_metrics.parquet" \
  -m all
echo -e "${GREEN}✓ Metrics computed${NC}"
echo ""

# Step 3: Bayes optimal (binary)
echo -e "${BLUE}[STEP 3]${NC} Computing Bayes optimal (binary, t=9)..."
uv run cli.py experiments-bin bayes-optimal \
  --input "$OUTPUT_DIR/npn_subset_metrics.parquet" \
  --target min_gates \
  --binary \
  --threshold 9 \
  --metrics entanglement,entropy,spectral_entropy,sensitivity \
  --output-csv "$OUTPUT_DIR/bayes_optimal_subset_t9.csv"
echo -e "${GREEN}✓ Bayes analysis complete${NC}"
echo ""

# Step 4: Orthogonality 
echo -e "${BLUE}[STEP 4]${NC} Computing orthogonality (4 core metrics)..."
uv run cli.py experiments-bin orthogonality \
  --input "$OUTPUT_DIR/npn_subset_metrics.parquet" \
  --target min_gates \
  --binary \
  --threshold 9 \
  --metrics entanglement,entropy,spectral_entropy,sensitivity \
  --output-csv "$OUTPUT_DIR/orthogonality_subset_t9.csv" \
  --output-mi-csv "$OUTPUT_DIR/mi_subset_t9.csv"
echo -e "${GREEN}✓ Orthogonality analysis complete${NC}"
echo ""

# Show results
echo "=========================================="
echo -e "${GREEN}Smoke Test Complete!${NC}"
echo "=========================================="
echo ""
echo "Generated files (subset, ~100K functions):"
ls -lh "$OUTPUT_DIR"/*.{parquet,csv} 2>/dev/null | awk '{print "  " $NF " (" $5 ")"}'
echo ""
echo "Key results:"
echo "  • Bayes optimal: $OUTPUT_DIR/bayes_optimal_subset_t9.csv"
echo ""
cat "$OUTPUT_DIR/bayes_optimal_subset_t9.csv"
echo ""
echo "Next steps:"
echo "  1. Review the CSV results to verify workflow"
echo "  2. Run full pipeline when ready:"
echo "     cd experiments && bash scripts/full_pipeline.sh _results"
echo ""
