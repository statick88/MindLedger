#!/bin/bash
# run-full-audit.sh — Orchestrate the complete security audit pipeline
# Part of the comprehensive security audit for MindLedger
# Runs all audit scripts in sequence and generates final report
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "╔══════════════════════════════════════════════════╗"
echo "║  MindLedger — Full Security Audit Pipeline       ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "Project root: $PROJECT_ROOT"
echo "Started at:   $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

ERRORS=0

# --- Phase 5: Dependency Audit ---
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 5: Dependency Audit"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash "$SCRIPT_DIR/dep-audit.sh"; then
    echo "✓ Phase 5 complete"
else
    echo "✗ Phase 5 failed"
    ERRORS=$((ERRORS + 1))
fi
echo ""

if bash "$SCRIPT_DIR/dep-audit-report.sh"; then
    echo "✓ Phase 5 report complete"
else
    echo "✗ Phase 5 report failed"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# --- Phase 6: SAST ---
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 6: Static Analysis (SAST)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

for script in sast-clippy.sh sast-unsafe-audit.sh sast-error-leakage.sh; do
    echo ""
    echo "--- Running $script ---"
    if bash "$SCRIPT_DIR/$script"; then
        echo "✓ $script complete"
    else
        echo "✗ $script failed"
        ERRORS=$((ERRORS + 1))
    fi
done
echo ""

# --- Phase 8: Verification ---
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 8: Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo ""
echo "[cargo test] Running workspace tests..."
if command -v cargo &>/dev/null; then
    cargo test --workspace 2>&1 | tail -20 || {
        echo "✗ cargo test failed"
        ERRORS=$((ERRORS + 1))
    }
else
    echo "⚠ cargo not available — skipping tests"
fi
echo ""

# --- Phase 7: Report Generation ---
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 7: Report Generation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash "$SCRIPT_DIR/generate-audit-report.sh"; then
    echo "✓ Report generated"
else
    echo "✗ Report generation failed"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# --- Summary ---
echo "╔══════════════════════════════════════════════════╗"
echo "║  Audit Pipeline Complete                         ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "Completed at: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

if [ "$ERRORS" -gt 0 ]; then
    echo "⚠ $ERRORS script(s) encountered errors"
    echo "  Check audit-output/ for details"
    exit 1
else
    echo "✓ All scripts completed successfully"
    echo ""
    echo "Output files:"
    ls -la "$PROJECT_ROOT/audit-output/" 2>/dev/null || echo "  (no output files)"
    echo ""
    echo "Report: $PROJECT_ROOT/sdd-archive/SECURITY-AUDIT-REPORT.md"
    exit 0
fi
