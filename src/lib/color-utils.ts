/**
 * HEX to HSL conversion utility for CSS variable injection.
 * Converts tenant brand colors (HEX) to HSL format for Tailwind CSS variables.
 * 
 * Output format: "h s% l%" (e.g., "192 72% 21%")
 * Matches Tailwind's hsl(var(--variable)) expectation.
 */

import type { TypographyConfig } from '@/types/tenant';

/**
 * Convert a HEX color string to HSL format for CSS variables.
 * 
 * @param hex - HEX color string (e.g., "#1A5F60" or "1A5F60")
 * @returns HSL string in format "h s% l%" (e.g., "192 72% 21%")
 * @throws Error if hex format is invalid
 */
export function hexToHsl(hex: string): string {
  // Remove # if present
  const cleanHex = hex.startsWith('#') ? hex.slice(1) : hex;
  
  // Validate hex format
  if (!/^[0-9A-Fa-f]{6}$/.test(cleanHex)) {
    throw new Error(`Invalid HEX color format: ${hex}. Expected 6-digit hex (e.g., #1A5F60)`);
  }

  // Parse RGB values (0-255)
  const r = parseInt(cleanHex.slice(0, 2), 16) / 255;
  const g = parseInt(cleanHex.slice(2, 4), 16) / 255;
  const b = parseInt(cleanHex.slice(4, 6), 16) / 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  let h = 0;
  let s = 0;
  const l = (max + min) / 2;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    
    switch (max) {
      case r:
        h = (g - b) / d + (g < b ? 6 : 0);
        break;
      case g:
        h = (b - r) / d + 2;
        break;
      case b:
        h = (r - g) / d + 4;
        break;
    }
    h *= 60;
  }

  // Return HSL as "h s% l%" (degrees, percentage, percentage)
  return `${Math.round(h)} ${Math.round(s * 100)}% ${Math.round(l * 100)}%`;
}

/**
 * Convert all brand tokens from HEX to HSL.
 * Useful for batch conversion when injecting CSS variables.
 * 
 * @param brand - BrandTokens object with HEX values
 * @returns BrandTokens with HSL string values
 */
export function brandTokensToHsl(brand: Record<string, string>): Record<string, string> {
  const result: Record<string, string> = {};
  for (const [key, value] of Object.entries(brand)) {
    result[key] = hexToHsl(value);
  }
  return result;
}

/**
 * Convert camelCase to kebab-case for CSS variable names.
 * e.g., "primaryForeground" -> "primary-foreground"
 */
export function kebabCase(str: string): string {
  return str.replace(/([a-z])([A-Z])/g, '$1-$2').toLowerCase();
}

/**
 * Inject tenant brand tokens as CSS variables on the document root.
 * 
 * @param root - Document root element (typically document.documentElement)
 * @param brand - Light mode brand tokens (HEX)
 * @param brandDark - Dark mode brand tokens (HEX)
 * @param typography - Typography configuration
 */
export function injectCssVariables(
  root: HTMLElement,
  brand: Record<string, string>,
  brandDark: Record<string, string>,
  typography: TypographyConfig
): void {
  // Light mode variables
  for (const [key, value] of Object.entries(brand)) {
    root.style.setProperty(`--${kebabCase(key)}`, hexToHsl(value));
  }

  // Dark mode variables (stored as --*-dark, activated via .dark selector in CSS)
  for (const [key, value] of Object.entries(brandDark)) {
    root.style.setProperty(`--${kebabCase(key)}-dark`, hexToHsl(value));
  }

  // Typography variables
  root.style.setProperty('--font-family', typography.fontFamily);
  root.style.setProperty('--heading-weight', typography.headingWeight);
  root.style.setProperty('--body-weight', typography.bodyWeight);
}

// Re-export types for convenience
export type { TenantConfig, BrandTokens, TypographyConfig } from '@/types/tenant';