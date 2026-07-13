#!/bin/bash
# sast-unsafe-audit.sh — Parse cargo-geiger output for undocumented unsafe blocks
# Part of the comprehensive security audit for MindLedger
# Outputs JSON to audit-output/
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/audit-output"

mkdir -p "$OUTPUT_DIR"

echo "=== SAST: Unsafe Code Audit ==="
echo "Project root: $PROJECT_ROOT"
echo ""

GEIGER_JSON="$OUTPUT_DIR/cargo-geiger.json"
UNSAFE_REPORT="$OUTPUT_DIR/unsafe-audit.json"

if [ ! -f "$GEIGER_JSON" ]; then
    echo "⚠ cargo-geiger.json not found — run dep-audit.sh first"
    echo '{"tool":"cargo-geiger","error":"cargo-geiger.json not found","findings":[]}' > "$UNSAFE_REPORT"
    exit 0
fi

# Check if geiger was skipped
if python3 -c "import json; d=json.load(open('$GEIGER_JSON')); exit(0 if d.get('skipped') else 1)" 2>/dev/null; then
    echo "⚠ cargo-geiger was skipped — no data to analyze"
    echo '{"tool":"cargo-geiger","error":"skipped","findings":[]}' > "$UNSAFE_REPORT"
    exit 0
fi

echo "[1/2] Scanning Rust source files for unsafe blocks..."

# Find all unsafe blocks in Rust source files (excluding generated code and tests)
UNSAFE_BLOCKS=$(grep -rn "unsafe {" \
    --include="*.rs" \
    --exclude-dir=target \
    --exclude-dir=tests \
    --exclude="*_test.rs" \
    --exclude="mod.rs" \
    "$PROJECT_ROOT/src-tauri/" 2>/dev/null || true)

UNSAFE_COUNT=$(echo "$UNSAFE_BLOCKS" | grep -c "." || true)

echo "[2/2] Checking for SAFETY comments..."

# Check which unsafe blocks have SAFETY comments
DOCUMENTED=0
UNDOCUMENTED=0
UNDOCUMENTED_LIST=""

while IFS= read -r line; do
    [ -z "$line" ] && continue
    FILE=$(echo "$line" | cut -d: -f1)
    LINENUM=$(echo "$line" | cut -d: -f2)
    
    # Check if the line before or after has a // SAFETY: comment
    PREV_LINE=$((LINENUM - 1))
    NEXT_LINE=$((LINENUM + 1))
    
    HAS_SAFETY=false
    if sed -n "${PREV_LINE}p" "$FILE" 2>/dev/null | grep -qi "SAFETY:"; then
        HAS_SAFETY=true
    fi
    if sed -n "${NEXT_LINE}p" "$FILE" 2>/dev/null | grep -qi "SAFETY:"; then
        HAS_SAFETY=true
    fi
    # Also check inline
    if echo "$line" | grep -qi "SAFETY:"; then
        HAS_SAFETY=true
    fi
    
    if [ "$HAS_SAFETY" = true ]; then
        DOCUMENTED=$((DOCUMENTED + 1))
    else
        UNDOCUMENTED=$((UNDOCUMENTED + 1))
        REL_PATH="${FILE#$PROJECT_ROOT/}"
        UNDOCUMENTED_LIST="${UNDOCUMENTED_LIST}    {\"file\":\"$REL_PATH\",\"line\":$LINENUM},\n"
    fi
done <<< "$UNSAFE_BLOCKS"

# Remove trailing comma from list
UNDOCUMENTED_LIST=$(echo -e "$UNDOCUMENTED_LIST" | sed '$ s/,$//')

# Generate JSON report
cat > "$UNSAFE_REPORT" <<EOF
{
  "tool": "cargo-geiger + source-scan",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "summary": {
    "total_unsafe_blocks": $UNSAFE_COUNT,
    "documented": $DOCUMENTED,
    "undocumented": $UNDOCUMENTED,
    "coverage_percent": $([ "$UNSAFE_COUNT" -gt 0 ] && echo "scale=1; $DOCUMENTED * 100 / $UNSAFE_COUNT" | bc 2>/dev/null || echo "0")
  },
  "undocumented_findings": [
$UNDOCUMENTED_LIST
  ],
  "verdict": $([ "$UNDOCUMENTED" -eq 0 ] && echo '"PASS"' || echo '"FAIL"'),
  "spec_ref": "dependency-vulnerabilities/Scenario: cargo-geiger coverage"
}
EOF

echo ""
echo "=== Unsafe Code Audit Results ==="
echo "  Total unsafe blocks:  $UNSAFE_COUNT"
echo "  Documented (SAFETY):  $DOCUMENTED"
echo "  Undocumented:         $UNDOCUMENTED"
echo ""

if [ "$UNDOCUMENTED" -gt 0 ]; then
    echo "⚠ Undocumented unsafe blocks found:"
    echo -e "$UNDOCUMENTED_LIST"
    echo "Every unsafe block MUST have a // SAFETY: comment per spec."
else
    echo "✓ All unsafe blocks are documented"
fi

echo ""
echo "JSON report: $UNSAFE_REPORT"
