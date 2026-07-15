#!/bin/bash
# sast-error-leakage.sh — Scan AppError variants for information leakage
# Part of the comprehensive security audit for MindLedger
# Checks for file paths, SQL fragments, memory addresses in error messages
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/audit-output"
SRC_DIR="$PROJECT_ROOT/src-tauri"

mkdir -p "$OUTPUT_DIR"

echo "=== SAST: Error Leakage Audit ==="
echo "Source dir: $SRC_DIR"
echo ""

LEAKAGE_REPORT="$OUTPUT_DIR/error-leakage.json"
FINDINGS=""
FINDING_COUNT=0

# --- Check 1: AppError format strings containing file paths ---
echo "[1/4] Checking for file path leakage in error messages..."
PATH_HITS=$(grep -rn 'AppError\|\.to_string()' \
    --include="*.rs" \
    "$SRC_DIR/" 2>/dev/null \
    | grep -iE '(format!\(.*["\x27]/|\.display\(\)|\.to_string_lossy|std::path|PathBuf)' \
    | grep -v '#\[test\]' \
    | grep -v 'target/' || true)

PATH_COUNT=$(echo "$PATH_HITS" | grep -c "." || true)
if [ "$PATH_COUNT" -gt 0 ]; then
    echo "  ⚠ Found $PATH_COUNT potential file path leakages"
    while IFS= read -r hit; do
        [ -z "$hit" ] && continue
        FILE=$(echo "$hit" | cut -d: -f1)
        LINE=$(echo "$hit" | cut -d: -f2)
        REL="${FILE#$PROJECT_ROOT/}"
        FINDINGS="${FINDINGS}    {\"category\":\"file_path\",\"file\":\"$REL\",\"line\":$LINE,\"severity\":\"high\"},\n"
        FINDING_COUNT=$((FINDING_COUNT + 1))
    done <<< "$PATH_HITS"
else
    echo "  ✓ No file path leakage detected"
fi

# --- Check 2: SQL fragments in error messages ---
echo "[2/4] Checking for SQL fragment leakage..."
SQL_HITS=$(grep -rn 'AppError\|format!\|to_string()' \
    --include="*.rs" \
    "$SRC_DIR/" 2>/dev/null \
    | grep -iE '(SELECT|INSERT|UPDATE|DELETE|DROP|CREATE TABLE|FROM |WHERE |JOIN )' \
    | grep -v '#\[test\]' \
    | grep -v 'target/' \
    | grep -v '//' || true)

SQL_COUNT=$(echo "$SQL_HITS" | grep -c "." || true)
if [ "$SQL_COUNT" -gt 0 ]; then
    echo "  ⚠ Found $SQL_COUNT potential SQL fragment leakages"
    while IFS= read -r hit; do
        [ -z "$hit" ] && continue
        FILE=$(echo "$hit" | cut -d: -f1)
        LINE=$(echo "$hit" | cut -d: -f2)
        REL="${FILE#$PROJECT_ROOT/}"
        FINDINGS="${FINDINGS}    {\"category\":\"sql_fragment\",\"file\":\"$REL\",\"line\":$LINE,\"severity\":\"critical\"},\n"
        FINDING_COUNT=$((FINDING_COUNT + 1))
    done <<< "$SQL_HITS"
else
    echo "  ✓ No SQL fragment leakage detected"
fi

# --- Check 3: Memory addresses in error messages ---
echo "[3/4] Checking for memory address leakage..."
ADDR_HITS=$(grep -rn 'AppError\|format!\|e:.*to_string\|{:p}' \
    --include="*.rs" \
    "$SRC_DIR/" 2>/dev/null \
    | grep -iE '(\{:p\}|as \*const|as \*mut|ptr::|0x[0-9a-fA-F]+|memory address)' \
    | grep -v '#\[test\]' \
    | grep -v 'target/' || true)

ADDR_COUNT=$(echo "$ADDR_HITS" | grep -c "." || true)
if [ "$ADDR_COUNT" -gt 0 ]; then
    echo "  ⚠ Found $ADDR_COUNT potential memory address leakages"
    while IFS= read -r hit; do
        [ -z "$hit" ] && continue
        FILE=$(echo "$hit" | cut -d: -f1)
        LINE=$(echo "$hit" | cut -d: -f2)
        REL="${FILE#$PROJECT_ROOT/}"
        FINDINGS="${FINDINGS}    {\"category\":\"memory_address\",\"file\":\"$REL\",\"line\":$LINE,\"severity\":\"high\"},\n"
        FINDING_COUNT=$((FINDING_COUNT + 1))
    done <<< "$ADDR_HITS"
else
    echo "  ✓ No memory address leakage detected"
fi

# --- Check 4: Stack trace / backtrace leakage ---
echo "[4/4] Checking for stack trace leakage..."
BT_HITS=$(grep -rn 'AppError\|format!\|to_string()' \
    --include="*.rs" \
    "$SRC_DIR/" 2>/dev/null \
    | grep -iE '(backtrace|stack_trace|traceback|std::backtrace)' \
    | grep -v '#\[test\]' \
    | grep -v 'target/' || true)

BT_COUNT=$(echo "$BT_HITS" | grep -c "." || true)
if [ "$BT_COUNT" -gt 0 ]; then
    echo "  ⚠ Found $BT_COUNT potential stack trace leakages"
    while IFS= read -r hit; do
        [ -z "$hit" ] && continue
        FILE=$(echo "$hit" | cut -d: -f1)
        LINE=$(echo "$hit" | cut -d: -f2)
        REL="${FILE#$PROJECT_ROOT/}"
        FINDINGS="${FINDINGS}    {\"category\":\"stack_trace\",\"file\":\"$REL\",\"line\":$LINE,\"severity\":\"medium\"},\n"
        FINDING_COUNT=$((FINDING_COUNT + 1))
    done <<< "$BT_HITS"
else
    echo "  ✓ No stack trace leakage detected"
fi

# Remove trailing comma from findings
FINDINGS=$(echo -e "$FINDINGS" | sed '$ s/,$//')

# Generate JSON report
cat > "$LEAKAGE_REPORT" <<EOF
{
  "tool": "sast-error-leakage",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "summary": {
    "total_findings": $FINDING_COUNT,
    "file_path_leakage": $PATH_COUNT,
    "sql_fragment_leakage": $SQL_COUNT,
    "memory_address_leakage": $ADDR_COUNT,
    "stack_trace_leakage": $BT_COUNT,
    "verdict": $([ "$FINDING_COUNT" -eq 0 ] && echo '"PASS"' || echo '"FAIL"')
  },
  "findings": [
$FINDINGS
  ],
  "spec_ref": "sast-static-analysis/Scenario: Error path sanitization"
}
EOF

echo ""
echo "=== Error Leakage Audit Results ==="
echo "  File path leakages:     $PATH_COUNT"
echo "  SQL fragment leakages:  $SQL_COUNT"
echo "  Memory address leakages: $ADDR_COUNT"
echo "  Stack trace leakages:   $BT_COUNT"
echo "  Total findings:         $FINDING_COUNT"
echo ""

if [ "$FINDING_COUNT" -gt 0 ]; then
    echo "⚠ Error leakage findings detected — review $LEAKAGE_REPORT"
else
    echo "✓ No error leakage detected in AppError variants"
fi

echo ""
echo "JSON report: $LEAKAGE_REPORT"
