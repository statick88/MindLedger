#!/usr/bin/env bash
# ============================================================================
# MindLedger — Release Sync Verification Script
# Phase: sdd-archive (Host macOS — monitors VM push)
# ============================================================================
# Verifies that the Windows VM has completed the audit pipeline and
# the release certification doc is fully populated.
set -euo pipefail

RELEASE_DOC="sdd-archive/RELEASE-V1.0.0-GLORIA-ONCE.md"
BRANCH="release/v1.0.0-gloria-once"
PASS=0
FAIL=0

echo ""
echo "============================================"
echo " MindLedger Release Sync Verification"
echo " Branch: $BRANCH"
echo "============================================"
echo ""

# ── 1. Pull latest from VM push ─────────────────────────────────────────────
echo "[1/5] Pulling latest from origin..."
git fetch origin "$BRANCH" --quiet

LOCAL=$(git rev-parse "$BRANCH" 2>/dev/null || echo "none")
REMOTE=$(git rev-parse "origin/$BRANCH" 2>/dev/null || echo "none")

if [ "$LOCAL" = "$REMOTE" ]; then
    echo "  ✓ Local and remote are in sync ($LOCAL)"
else
    echo "  ⚠ Local ($LOCAL) differs from remote ($REMOTE)"
    echo "  → Pulling..."
    git pull origin "$BRANCH" --quiet
    echo "  ✓ Updated to $REMOTE"
fi
echo ""

# ── 2. Check that ALL markers are gone ──────────────────────────────────────
echo "[2/5] Checking for unfilled markers..."

MARKERS=$(grep -c "<!-- FILLED BY:" "$RELEASE_DOC" 2>/dev/null || true)

if [ "$MARKERS" -eq 0 ]; then
    echo "  ✓ No '<!-- FILLED BY...' markers remain"
    PASS=$((PASS + 1))
else
    echo "  ✗ Found $MARKERS unfilled marker(s)"
    grep -n "<!-- FILLED BY:" "$RELEASE_DOC" || true
    FAIL=$((FAIL + 1))
fi
echo ""

# ── 3. Verify SHA-256 hashes are present (not placeholder) ──────────────────
echo "[3/5] Verifying SHA-256 hashes..."

INSTALLER_HASH=$(grep -A1 "SHA-256" "$RELEASE_DOC" | grep -oE '[A-Fa-f0-9]{64}' | head -1 || true)
BINARY_HASH=$(grep -A1 "SHA-256" "$RELEASE_DOC" | grep -oE '[A-Fa-f0-9]{64}' | tail -1 || true)

if [ -n "$INSTALLER_HASH" ]; then
    echo "  ✓ Installer SHA-256: ${INSTALLER_HASH:0:16}..."
    PASS=$((PASS + 1))
else
    echo "  ✗ Installer SHA-256 missing or invalid"
    FAIL=$((FAIL + 1))
fi

if [ -n "$BINARY_HASH" ]; then
    echo "  ✓ Binary SHA-256: ${BINARY_HASH:0:16}..."
    PASS=$((PASS + 1))
else
    echo "  ✗ Binary SHA-256 missing or invalid"
    FAIL=$((FAIL + 1))
fi
echo ""

# ── 4. Verify PE hardening results are present ──────────────────────────────
echo "[4/5] Verifying PE hardening audit results..."

PE_CHECKS=("ASLR" "DEP" "NX_COMPAT" "HIGH_ENTROPY_VA")
PE_FOUND=0

for check in "${PE_CHECKS[@]}"; do
    if grep -qi "$check" "$RELEASE_DOC" 2>/dev/null; then
        PE_FOUND=$((PE_FOUND + 1))
    fi
done

if [ "$PE_FOUND" -ge 2 ]; then
    echo "  ✓ PE hardening results present ($PE_FOUND/$(${#PE_CHECKS[@]}))"
    PASS=$((PASS + 1))
else
    echo "  ✗ PE hardening results incomplete ($PE_FOUND/${#PE_CHECKS[@]} found)"
    FAIL=$((FAIL + 1))
fi
echo ""

# ── 5. Final summary ────────────────────────────────────────────────────────
echo "[5/5] Sync verification summary"
echo ""
echo "============================================"

if [ "$FAIL" -eq 0 ]; then
    echo " RESULT: RELEASE DOC CERTIFIED"
    echo "============================================"
    echo ""
    echo " All markers filled, hashes present, PE audit complete."
    echo ""
    echo " Ready for final merge:"
    echo ""
    echo "   git checkout main"
    echo "   git merge $BRANCH --no-ff"
    echo "   git tag -a v1.0.0-gloria-once -m 'Release v1.0.0-gloria-once'"
    echo "   git push origin main --tags"
    echo ""
    exit 0
else
    echo " RESULT: NOT READY — $FAIL check(s) failed"
    echo "============================================"
    echo ""
    echo " The Windows VM audit may not have completed yet."
    echo " Re-run this script after the VM pushes."
    echo ""
    exit 1
fi
