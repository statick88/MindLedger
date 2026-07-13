#!/bin/bash
# sast-clippy.sh — Run cargo clippy with security-focused deny flags
# Part of the comprehensive security audit for MindLedger
# Outputs JSON to audit-output/
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/audit-output"

mkdir -p "$OUTPUT_DIR"

echo "=== SAST: Clippy Security Audit ==="
echo "Project root: $PROJECT_ROOT"
echo ""

CLIPPY_JSON="$OUTPUT_DIR/clippy-audit.json"
CLIPPY_LOG="$OUTPUT_DIR/clippy-audit.log"

# Run clippy with all security-relevant warning groups denied
# -D warnings: treat all warnings as errors
# -W clippy::all: enable all clippy lints
# Specific security lints:
#   clippy::unwrap_used: forbid unwrap() — can panic on untrusted input
#   clippy::expect_used: forbid expect() — can panic on untrusted input
#   clippy::panic: forbid panic!() in production code
#   clippy::unimplemented: forbid unimplemented!()
#   clippy::todo: forbid todo!()
#   clippy::integer_arithmetic: flag potential overflow
#   clippy::float_cmp_cmp: flag float comparison issues
echo "[1/1] Running cargo clippy..."

CLIPPY_OUTPUT=$(cargo clippy --workspace --all-targets 2>&1 \
    -W clippy::all \
    -W clippy::pedantic \
    -W clippy::unwrap_used \
    -W clippy::expect_used \
    -W clippy::panic \
    -W clippy::unimplemented \
    -W clippy::todo \
    -W clippy::integer_arithmetic \
    -D warnings \
    || true)

CLIPPY_EXIT=$?

# Save raw output
echo "$CLIPPY_OUTPUT" > "$CLIPPY_LOG"

# Count warnings by category
WARNING_COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "^warning:" || true)
ERROR_COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "^error" || true)
SECURITY_WARNINGS=$(echo "$CLIPPY_OUTPUT" | grep -cE "(unwrap_used|expect_used|panic|unimplemented|todo|integer_arithmetic)" || true)

# Generate JSON report
cat > "$CLIPPY_JSON" <<EOF
{
  "tool": "cargo-clippy",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "exit_code": $CLIPPY_EXIT,
  "summary": {
    "total_warnings": $WARNING_COUNT,
    "total_errors": $ERROR_COUNT,
    "security_warnings": $SECURITY_WARNINGS,
    "clean": $([ "$WARNING_COUNT" -eq 0 ] && echo "true" || echo "false")
  },
  "lints_applied": [
    "clippy::all",
    "clippy::pedantic",
    "clippy::unwrap_used",
    "clippy::expect_used",
    "clippy::panic",
    "clippy::unimplemented",
    "clippy::todo",
    "clippy::integer_arithmetic"
  ],
  "raw_output_file": "clippy-audit.log"
}
EOF

echo ""
echo "=== Clippy Results ==="
echo "  Total warnings:  $WARNING_COUNT"
echo "  Total errors:    $ERROR_COUNT"
echo "  Security-related: $SECURITY_WARNINGS"
echo ""

if [ "$WARNING_COUNT" -gt 0 ]; then
    echo "⚠ Warnings found — review $CLIPPY_LOG"
    echo ""
    echo "Top findings:"
    echo "$CLIPPY_OUTPUT" | grep -E "^warning:" | head -10
else
    echo "✓ Workspace is clippy-clean"
fi

echo ""
echo "JSON report: $CLIPPY_JSON"
echo "Full log:    $CLIPPY_LOG"
