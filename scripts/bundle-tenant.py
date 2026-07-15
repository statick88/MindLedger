#!/usr/bin/env python3
"""
MindLdger — White-Label Tenant Bundler
=======================================
Generates platform-specific builds from a tenant JSON definition.
No source code modification required.

Usage:
    python3 scripts/bundle-tenant.py tenants/mindledger.json
    python3 scripts/bundle-tenant.py tenants/mindledger.json --dry-run
    python3 scripts/bundle-tenant.py tenants/mindledger.json --skip-icons

Outputs (pre-build):
    - src-tauri/tauri.conf.json     (templated productName, identifier)
    - tenant-configs/{id}.json      (legacy format for build.rs + frontend IPC)
    - src/tenant.config.json        (static frontend fallback)
    - src-tauri/.env.{id}           (TENANT_CONFIG env for build.rs)
    - src-tauri/icons/              (platform icons regenerated from icon_source)

Author: Diego Medardo Saavedra García <Statick>
Date:   2026-07-13
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

# ── Constants ──────────────────────────────────────────────────────────────────

ROOT = Path(__file__).resolve().parent.parent
TAURI_DIR = ROOT / "src-tauri"
TAURI_CONF = TAURI_DIR / "tenant.conf.json"  # base template (pre-tenant)
TAURI_CONF_TARGET = TAURI_DIR / "tauri.conf.json"
TENANT_CONFIGS_DIR = ROOT / "tenant-configs"
SRC_DIR = ROOT / "src"
ICONS_DIR = TAURI_DIR / "icons"

# Required fields in tenant JSON
REQUIRED_FIELDS = [
    "tenant_id",
    "tenant_name",
    "clinical_role",
    "bundle_id",
    "app_data_dir_name",
    "keyring_id",
    "theme",
    "assets",
]

REQUIRED_THEME = ["primary", "background_sage", "accent_coral"]
REQUIRED_ASSETS = ["icon_source", "logo_source"]

# CSP policy — must never be modified by the bundler
CSP_STRICT = (
    "default-src 'self'; "
    "style-src 'self'; "
    "script-src 'self'; "
    "img-src 'self'; "
    "font-src 'self'; "
    "object-src 'none'; "
    "frame-src 'none'; "
    "connect-src 'self'; "
    "base-uri 'self'; "
    "form-action 'self'"
)


# ── Validation ─────────────────────────────────────────────────────────────────

class BundleError(Exception):
    """Raised when bundling fails."""


def validate_tenant_json(tenant: dict[str, Any]) -> list[str]:
    """Validate tenant JSON schema. Returns list of errors (empty = valid)."""
    errors: list[str] = []

    for field in REQUIRED_FIELDS:
        if field not in tenant:
            errors.append(f"Missing required field: {field}")

    if "theme" in tenant:
        for field in REQUIRED_THEME:
            if field not in tenant["theme"]:
                errors.append(f"Missing theme field: {field}")

    if "assets" in tenant:
        for field in REQUIRED_ASSETS:
            if field not in tenant["assets"]:
                errors.append(f"Missing assets field: {field}")

    # Validate bundle_id format
    bundle_id = tenant.get("bundle_id", "")
    if bundle_id and not all(c.isalnum() or c in ".-" for c in bundle_id):
        errors.append(f"Invalid bundle_id format: {bundle_id}")

    return errors


def validate_assets(tenant: dict[str, Any], dry_run: bool = False) -> list[str]:
    """Validate that asset files exist. Returns list of errors."""
    errors: list[str] = []
    assets = tenant.get("assets", {})

    for key, path_str in assets.items():
        asset_path = ROOT / path_str
        if not asset_path.exists():
            errors.append(f"Asset not found: {path_str}")
        elif not dry_run:
            # Validate image dimensions for icon
            if key == "icon_source":
                try:
                    from PIL import Image
                    img = Image.open(asset_path)
                    w, h = img.size
                    if w < 256 or h < 256:
                        errors.append(f"Icon too small ({w}x{h}), minimum 256x256")
                    if w != h:
                        errors.append(f"Icon not square ({w}x{h})")
                except ImportError:
                    pass  # PIL not available, skip dimension check

    return errors


# ── Tauri Config Templating ───────────────────────────────────────────────────

def template_tauri_conf(tenant: dict[str, Any], dry_run: bool = False) -> dict[str, Any]:
    """
    Template tauri.conf.json with tenant-specific values.
    Preserves CSP and all security-critical fields.
    """
    # Read base template
    if TAURI_CONF.exists():
        conf_path = TAURI_CONF
    elif TAURI_CONF_TARGET.exists():
        conf_path = TAURI_CONF_TARGET
    else:
        raise BundleError(f"No tauri config found at {TAURI_CONF} or {TAURI_CONF_TARGET}")

    with open(conf_path) as f:
        conf = json.load(f)

    # Store original CSP for verification
    original_csp = conf.get("app", {}).get("security", {}).get("csp", "")

    # Apply tenant values
    tenant_name = tenant["tenant_name"]
    bundle_id = tenant["bundle_id"]
    tenant_id = tenant["tenant_id"]

    conf["productName"] = tenant_name
    conf["identifier"] = bundle_id

    # Window title
    if "app" in conf and "windows" in conf["app"] and conf["app"]["windows"]:
        conf["app"]["windows"][0]["title"] = tenant_name

    # Long description
    if "bundle" in conf:
        conf["bundle"]["longDescription"] = (
            f"{tenant_name} — {tenant.get('clinical_role', 'Clinical Practice Management')}. "
            f"Secure, offline-first practice management system."
        )

    # Verify CSP is preserved
    new_csp = conf.get("app", {}).get("security", {}).get("csp", "")
    if original_csp and new_csp != original_csp:
        raise BundleError("CSP was modified during templating — this is a security violation")

    if not dry_run:
        with open(TAURI_CONF_TARGET, "w") as f:
            json.dump(conf, f, indent=2, ensure_ascii=False)
            f.write("\n")
        print(f"  ✓ tauri.conf.json templated → {bundle_id}")

    return conf


# ── Legacy Config Generation ──────────────────────────────────────────────────

def generate_legacy_config(tenant: dict[str, Any], dry_run: bool = False) -> Path:
    """
    Generate tenant-configs/{id}.json in the legacy format expected by
    build.rs and the frontend IPC layer (get_tenant_config command).
    """
    tenant_id = tenant["tenant_id"]
    theme = tenant.get("theme", {})

    # Map new theme fields to legacy brand tokens
    legacy = {
        "tenant": {
            "id": tenant_id,
            "commercialName": tenant["tenant_name"],
            "clinicalRole": tenant.get("clinical_role", ""),
            "ownerName": tenant.get("tenant_name", ""),
            "ownerTitle": tenant.get("clinical_role", ""),
        },
        "brand": {
            "primary": theme.get("primary", "#1A5F60"),
            "primaryForeground": "#FFFFFF",
            "secondary": theme.get("background_sage", "#E5F1EE"),
            "secondaryForeground": "#212529",
            "accent": theme.get("accent_coral", "#E3645F"),
            "accentForeground": "#FFFFFF",
            "background": "#F8F9FA",
            "foreground": "#212529",
            "muted": "#F0F2F5",
            "mutedForeground": "#6B7280",
            "card": "#FFFFFF",
            "cardForeground": "#212529",
            "border": "#DEE2E6",
            "input": "#DEE2E6",
            "ring": theme.get("primary", "#1A5F60"),
            "destructive": "#DC3545",
            "destructiveForeground": "#FFFFFF",
        },
        "brandDark": {
            "primary": _lighten(theme.get("primary", "#1A5F60"), 0.2),
            "primaryForeground": "#FFFFFF",
            "secondary": "#1E3A3A",
            "secondaryForeground": "#E5F1EE",
            "accent": _lighten(theme.get("accent_coral", "#E3645F"), -0.1),
            "accentForeground": "#FFFFFF",
            "background": "#1A1D21",
            "foreground": "#F0F2F5",
            "muted": "#2A2D31",
            "mutedForeground": "#9CA3AF",
            "card": "#222529",
            "cardForeground": "#F0F2F5",
            "border": "#333639",
            "input": "#333639",
            "ring": _lighten(theme.get("primary", "#1A5F60"), 0.2),
            "destructive": "#B82E2E",
            "destructiveForeground": "#FFFFFF",
        },
        "typography": {
            "fontFamily": "'Inter', system-ui, -apple-system, sans-serif",
            "headingWeight": "700",
            "bodyWeight": "400",
        },
        "crypto": {
            "keyringService": "mind-ledger",
            "keyringAccount": tenant.get("keyring_id", "sqlcipher-key"),
            "dbFileName": f"mind_ledger_{tenant_id.replace('-', '_')}.db",
        },
        "features": {
            "clinicalNotes": True,
            "accounting": True,
            "agenda": True,
            "diagnostics": True,
        },
    }

    dest = TENANT_CONFIGS_DIR / f"{tenant_id}.json"
    if not dry_run:
        TENANT_CONFIGS_DIR.mkdir(parents=True, exist_ok=True)
        with open(dest, "w") as f:
            json.dump(legacy, f, indent=2, ensure_ascii=False)
            f.write("\n")
        print(f"  ✓ Legacy config → {dest.relative_to(ROOT)}")

    return dest


# ── Frontend Config Generation ────────────────────────────────────────────────

def generate_frontend_config(tenant: dict[str, Any], dry_run: bool = False) -> Path:
    """
    Generate src/tenant.config.json — a static fallback for the frontend.
    In production, the frontend reads from Tauri IPC (get_tenant_config).
    This file serves as type-safe reference and dev fallback.
    """
    tenant_id = tenant["tenant_id"]
    theme = tenant.get("theme", {})

    frontend_config = {
        "_meta": {
            "generated_by": "bundle-tenant.py",
            "tenant_id": tenant_id,
            "do_not_edit": True,
        },
        "tenant": {
            "id": tenant_id,
            "name": tenant["tenant_name"],
            "role": tenant.get("clinical_role", ""),
        },
        "theme": {
            "primary": theme.get("primary", "#1A5F60"),
            "background_sage": theme.get("background_sage", "#E5F1EE"),
            "accent_coral": theme.get("accent_coral", "#E3645F"),
        },
        "appDataDir": tenant.get("app_data_dir_name", f"mind-ledger-{tenant_id}"),
    }

    dest = SRC_DIR / "tenant.config.json"
    if not dry_run:
        with open(dest, "w") as f:
            json.dump(frontend_config, f, indent=2, ensure_ascii=False)
            f.write("\n")
        print(f"  ✓ Frontend config → {dest.relative_to(ROOT)}")

    return dest


# ── Environment Variables ─────────────────────────────────────────────────────

def generate_env_file(tenant: dict[str, Any], dry_run: bool = False) -> Path:
    """
    Generate src-tauri/.env.{tenant_id} with build-time env vars.
    build.rs reads TENANT_CONFIG from this file.
    """
    tenant_id = tenant["tenant_id"]
    legacy_config = TENANT_CONFIGS_DIR / f"{tenant_id}.json"

    env_content = f"""# Auto-generated by bundle-tenant.py — DO NOT EDIT
# Tenant: {tenant['tenant_name']}
# Run: python3 scripts/bundle-tenant.py tenants/{tenant_id.replace('_', '/')}.json
TENANT_CONFIG={legacy_config}
TENANT_ID={tenant_id}
TENANT_BUNDLE_ID={tenant['bundle_id']}
TENANT_APP_DATA_DIR={tenant.get('app_data_dir_name', f'mind-ledger-{tenant_id}')}
TENANT_KEYRING_ID={tenant.get('keyring_id', 'sqlcipher-key')}
"""

    # Generate .env (read by dotenvy in build.rs) + .env.{id} for reference
    env_dot = TAURI_DIR / ".env"
    env_tenant = TAURI_DIR / f".env.{tenant_id}"

    if not dry_run:
        with open(env_dot, "w") as f:
            f.write(env_content)
        with open(env_tenant, "w") as f:
            f.write(env_content)
        print(f"  ✓ Env file → {env_dot.relative_to(ROOT)} (+ .env.{tenant_id})")

    return env_dot


# ── Icon Generation ───────────────────────────────────────────────────────────

def generate_platform_icons(tenant: dict[str, Any], skip: bool = False, dry_run: bool = False) -> None:
    """
    Generate platform-specific icons from the tenant's icon_source.
    Uses Tauri CLI if available, falls back to PIL.
    """
    if skip:
        print("  ⏭ Icon generation skipped (--skip-icons)")
        return

    icon_source = ROOT / tenant["assets"]["icon_source"]
    if not icon_source.exists():
        print(f"  ⚠ Icon source not found: {tenant['assets']['icon_source']}")
        return

    # Try Tauri CLI first
    tauri_cli = shutil.which("tauri") or shutil.which("pnpm")
    if tauri_cli:
        try:
            cmd = ["pnpm", "tauri", "icon", str(icon_source)]
            result = subprocess.run(
                cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=60
            )
            if result.returncode == 0:
                print(f"  ✓ Platform icons generated via Tauri CLI")
                return
        except (subprocess.TimeoutExpired, FileNotFoundError):
            pass

    # Fallback: PIL-based icon generation
    try:
        from PIL import Image

        img = Image.open(icon_source)
        if img.mode != "RGBA":
            img = img.convert("RGBA")

        sizes = {
            "32x32.png": 32,
            "64x64.png": 64,
            "128x128.png": 128,
            "128x128@2x.png": 256,
            "icon.png": 512,
            # Windows Store icons
            "Square30x30Logo.png": 30,
            "Square44x44Logo.png": 44,
            "Square71x71Logo.png": 71,
            "Square89x89Logo.png": 89,
            "Square107x107Logo.png": 107,
            "Square142x142Logo.png": 142,
            "Square150x150Logo.png": 150,
            "Square284x284Logo.png": 284,
            "Square310x310Logo.png": 310,
            "StoreLogo.png": 50,
        }

        if not dry_run:
            for name, size in sizes.items():
                resized = img.resize((size, size), Image.Resampling.LANCZOS)
                resized.save(ICONS_DIR / name)

            # Generate .icns for macOS
            try:
                subprocess.run(
                    ["png2icns", str(ICONS_DIR / "icon.icns"),
                     f"{ICONS_DIR}/icon.png=64",
                     f"{ICONS_DIR}/128x128.png=128",
                     f"{ICONS_DIR}/128x128@2x.png=256",
                     f"{ICONS_DIR}/icon.png=512"],
                    capture_output=True, timeout=30
                )
            except (FileNotFoundError, subprocess.TimeoutExpired):
                pass  # png2icns not available, skip .icns

            # Generate .ico for Windows
            try:
                ico_sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
                ico_images = [img.resize(s, Image.Resampling.LANCZOS) for s in ico_sizes]
                ico_images[0].save(
                    ICONS_DIR / "icon.ico",
                    format="ICO",
                    sizes=ico_sizes,
                    append_images=ico_images[1:],
                )
            except Exception:
                pass  # ICO generation failed, skip

            print(f"  ✓ Platform icons generated via PIL ({len(sizes)} sizes)")
        else:
            print(f"  ⏭ Icon generation skipped (dry-run)")

    except ImportError:
        print("  ⚠ PIL not available — install Pillow for icon generation: pip install Pillow")


# ── Helpers ────────────────────────────────────────────────────────────────────

def _lighten(hex_color: str, factor: float) -> str:
    """Lighten (positive) or darken (negative) a hex color."""
    hex_color = hex_color.lstrip("#")
    r, g, b = int(hex_color[0:2], 16), int(hex_color[2:4], 16), int(hex_color[4:6], 16)

    if factor > 0:
        r = min(255, int(r + (255 - r) * factor))
        g = min(255, int(g + (255 - g) * factor))
        b = min(255, int(b + (255 - b) * factor))
    else:
        f = abs(factor)
        r = max(0, int(r * (1 - f)))
        g = max(0, int(g * (1 - f)))
        b = max(0, int(b * (1 - f)))

    return f"#{r:02x}{g:02x}{b:02x}"


def print_summary(tenant: dict[str, Any]) -> None:
    """Print bundle summary."""
    print()
    print("=" * 60)
    print(f"  TENANT BUNDLE SUMMARY")
    print("=" * 60)
    print(f"  Tenant:      {tenant['tenant_name']}")
    print(f"  ID:          {tenant['tenant_id']}")
    print(f"  Bundle ID:   {tenant['bundle_id']}")
    print(f"  Data Dir:    {tenant.get('app_data_dir_name', 'N/A')}")
    print(f"  Keyring:     {tenant.get('keyring_id', 'N/A')}")
    print(f"  Theme:       {tenant.get('theme', {}).get('primary', 'N/A')}")
    print("=" * 60)
    print()
    print("  Next steps:")
    print("    1. pnpm build          # Build frontend")
    print("    2. cargo check          # Verify Rust backend")
    print("    3. pnpm tauri build     # Full release build")
    print()


# ── Main ───────────────────────────────────────────────────────────────────────

def main() -> int:
    parser = argparse.ArgumentParser(
        description="MindLdger White-Label Tenant Bundler",
        epilog="Example: python3 scripts/bundle-tenant.py tenants/gloria_once.json",
    )
    parser.add_argument(
        "tenant_json",
        help="Path to tenant JSON configuration file",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Validate only, don't write files",
    )
    parser.add_argument(
        "--skip-icons",
        action="store_true",
        help="Skip platform icon generation",
    )
    args = parser.parse_args()

    # Resolve tenant JSON path
    tenant_path = Path(args.tenant_json)
    if not tenant_path.is_absolute():
        tenant_path = ROOT / tenant_path

    print()
    print("MindLdger — White-Label Tenant Bundler")
    print("-" * 40)

    # ── Step 1: Load and validate tenant JSON ──
    print("[1/5] Validating tenant configuration...")
    if not tenant_path.exists():
        print(f"  ✗ Tenant file not found: {tenant_path}")
        return 1

    try:
        with open(tenant_path) as f:
            tenant = json.load(f)
    except json.JSONDecodeError as e:
        print(f"  ✗ Invalid JSON: {e}")
        return 1

    errors = validate_tenant_json(tenant)
    if errors:
        for err in errors:
            print(f"  ✗ {err}")
        return 1
    print(f"  ✓ Schema valid: {tenant['tenant_id']}")

    # ── Step 2: Validate assets ──
    print("[2/5] Validating assets...")
    asset_errors = validate_assets(tenant, dry_run=args.dry_run)
    if asset_errors:
        for err in asset_errors:
            print(f"  ✗ {err}")
        return 1
    print(f"  ✓ All assets present")

    # ── Step 3: Template tauri.conf.json ──
    print("[3/5] Templating tauri.conf.json...")
    try:
        template_tauri_conf(tenant, dry_run=args.dry_run)
    except BundleError as e:
        print(f"  ✗ {e}")
        return 1

    # ── Step 4: Generate configs ──
    print("[4/5] Generating configuration files...")
    generate_legacy_config(tenant, dry_run=args.dry_run)
    generate_frontend_config(tenant, dry_run=args.dry_run)
    generate_env_file(tenant, dry_run=args.dry_run)

    # ── Step 5: Generate icons ──
    print("[5/5] Generating platform icons...")
    generate_platform_icons(tenant, skip=args.skip_icons, dry_run=args.dry_run)

    # ── Summary ──
    if not args.dry_run:
        print_summary(tenant)

    return 0


if __name__ == "__main__":
    sys.exit(main())
