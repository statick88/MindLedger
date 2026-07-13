#!/bin/bash
# dep-audit.sh — Run dependency vulnerability audits for Rust and JavaScript
# Part of the comprehensive security audit for MindLedger
# Outputs JSON to audit-output/ directory
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/audit-output"

mkdir -p "$OUTPUT_DIR"

echo "=== Dependency Audit ==="
echo "Project root: $PROJECT_ROOT"
echo "Output dir:   $OUTPUT_DIR"
echo ""

# --- Rust: cargo audit ---
echo "[1/3] Running cargo audit..."
if command -v cargo-audit &>/dev/null || cargo audit --version &>/dev/null 2>&1; then
    cargo audit --json > "$OUTPUT_DIR/cargo-audit.json" 2>"$OUTPUT_DIR/cargo-audit-stderr.log" \
        && echo "  ✓ cargo-audit completed — $OUTPUT_DIR/cargo-audit.json" \
        || echo "  ✗ cargo-audit exited with errors (check $OUTPUT_DIR/cargo-audit-stderr.log)"
else
    echo "  ⚠ cargo-audit not installed — skipping"
    echo '{"error":"cargo-audit not installed","skipped":true}' > "$OUTPUT_DIR/cargo-audit.json"
fi

# --- Rust: cargo geiger ---
echo "[2/3] Running cargo geiger..."
if command -v cargo-geiger &>/dev/null || cargo geiger --version &>/dev/null 2>&1; then
    cargo geiger --output-format json > "$OUTPUT_DIR/cargo-geiger.json" 2>"$OUTPUT_DIR/cargo-geiger-stderr.log" \
        && echo "  ✓ cargo-geiger completed — $OUTPUT_DIR/cargo-geiger.json" \
        || echo "  ✗ cargo-geiger exited with errors (check $OUTPUT_DIR/cargo-geiger-stderr.log)"
else
    echo "  ⚠ cargo-geiger not installed — skipping"
    echo '{"error":"cargo-geiger not installed","skipped":true}' > "$OUTPUT_DIR/cargo-geiger.json"
fi

# --- JavaScript: pnpm audit ---
echo "[3/3] Running pnpm audit..."
if command -v pnpm &>/dev/null; then
    pnpm audit --json > "$OUTPUT_DIR/pnpm-audit.json" 2>"$OUTPUT_DIR/pnpm-audit-stderr.log" \
        && echo "  ✓ pnpm audit completed — $OUTPUT_DIR/pnpm-audit.json" \
        || echo "  ✗ pnpm audit exited with vulnerabilities (check $OUTPUT_DIR/pnpm-audit.json)"
else
    echo "  ⚠ pnpm not installed — skipping"
    echo '{"error":"pnpm not installed","skipped":true}' > "$OUTPUT_DIR/pnpm-audit.json"
fi

echo ""
echo "=== Dependency Audit Complete ==="
echo "Results written to $OUTPUT_DIR/"
ls -la "$OUTPUT_DIR/"
