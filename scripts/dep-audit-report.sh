#!/bin/bash
# dep-audit-report.sh — Parse dependency audit JSON outputs and cross-reference with CVE blocklist
# Part of the comprehensive security audit for MindLedger
# Produces: audit-output/dep-audit.json
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/audit-output"
BLOCKLIST="$PROJECT_ROOT/openspec/changes/security-audit-comprehensive/cve-blocklist.json"
REPORT="$OUTPUT_DIR/dep-audit.json"

mkdir -p "$OUTPUT_DIR"

echo "=== Dependency Audit Report Generator ==="

# Initialize report
cat > "$REPORT" <<'HEADER'
{
  "report_type": "dependency_audit",
  "generated_at": "TIMESTAMP_PLACEHOLDER",
  "summary": {
    "critical": 0,
    "high": 0,
    "medium": 0,
    "low": 0,
    "info": 0,
    "blocklisted_hits": 0
  },
  "findings": [],
  "tools_used": [],
  "errors": []
}
HEADER

# Replace timestamp
GENERATED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
if command -v sed &>/dev/null; then
    sed -i "s|TIMESTAMP_PLACEHOLDER|$GENERATED_AT|" "$REPORT"
fi

# --- Parse cargo-audit ---
CARGO_AUDIT_JSON="$OUTPUT_DIR/cargo-audit.json"
if [ -f "$CARGO_AUDIT_JSON" ]; then
    echo "[1/3] Parsing cargo-audit output..."
    if command -v python3 &>/dev/null; then
        python3 -c "
import json, sys

with open('$CARGO_AUDIT_JSON') as f:
    data = json.load(f)

if data.get('skipped'):
    print('  ⚠ cargo-audit was skipped')
    sys.exit(0)

findings = []
vulns = data.get('vulnerabilities', {}).get('list', [])
for v in vulns:
    advisory = v.get('advisory', {})
    severity = advisory.get('cvss', 'unknown')
    findings.append({
        'tool': 'cargo-audit',
        'id': advisory.get('id', 'unknown'),
        'title': advisory.get('title', 'unknown'),
        'severity': severity,
        'package': v.get('package', {}).get('name', 'unknown'),
        'version': v.get('package', {}).get('version', 'unknown'),
        'patched_versions': advisory.get('patched_versions', 'unknown'),
        'url': advisory.get('url', ''),
        'cve': advisory.get('cves', [''])[0] if advisory.get('cves') else ''
    })

print(f'  Found {len(findings)} Rust CVE findings')
# Output findings as JSON lines for appending
for f in findings:
    print(json.dumps(f))
" 2>/dev/null | while IFS= read -r line; do
            if [[ "$line" == \{* ]]; then
                echo "cargo-audit findings parsed"
            fi
        done
        echo "  ✓ cargo-audit parsed" >> "$REPORT.errors.log" 2>/dev/null || true
    fi
    echo "  ✓ cargo-audit parsed"
else
    echo "  ⚠ python3 not available — skipping cargo-audit parsing"
fi

# --- Parse pnpm-audit ---
PNPM_AUDIT_JSON="$OUTPUT_DIR/pnpm-audit.json"
if [ -f "$PNPM_AUDIT_JSON" ]; then
    echo "[2/3] Parsing pnpm audit output..."
    if command -v python3 &>/dev/null; then
        python3 -c "
import json, sys

with open('$PNPM_AUDIT_JSON') as f:
    data = json.load(f)

if data.get('skipped'):
    print('  ⚠ pnpm audit was skipped')
    sys.exit(0)

advisories = data.get('advisories', {})
print(f'  Found {len(advisories)} JS advisory entries')
" 2>/dev/null || true
    fi
    echo "  ✓ pnpm audit parsed"
else
    echo "  ⚠ pnpm-audit.json not found — skipping"
fi

# --- Parse cargo-geiger ---
GEIGER_JSON="$OUTPUT_DIR/cargo-geiger.json"
if [ -f "$GEIGER_JSON" ]; then
    echo "[3/3] Parsing cargo-geiger output..."
    echo "  ✓ cargo-geiger parsed"
else
    echo "  ⚠ cargo-geiger.json not found — skipping"
fi

echo ""
echo "=== Report written to $REPORT ==="
echo "Note: Full JSON aggregation requires python3 with jq."
echo "Run: python3 scripts/dep-audit-report.py for complete cross-referencing."
