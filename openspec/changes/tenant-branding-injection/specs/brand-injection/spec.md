# Brand Injection — Full Spec (New Capability)

## Purpose

Defines the runtime CSS variable injection system that transforms tenant brand tokens into dynamic UI theming via Tailwind CSS integration.

## Requirements

### REQ-BI-001: CSS Variable Injection on Config Load
The frontend SHALL inject all `brand` and `brandDark` color tokens as CSS custom properties on `:root` and `.dark` selectors respectively when tenant config loads.

**Light Mode Injection (`:root`):**
```typescript
const lightVars = {
  '--primary': hsl(config.brand.primary),
  '--primary-foreground': hsl(config.brand.primaryForeground),
  '--secondary': hsl(config.brand.secondary),
  '--secondary-foreground': hsl(config.brand.secondaryForeground),
  '--accent': hsl(config.brand.accent),
  '--accent-foreground': hsl(config.brand.accentForeground),
  '--background': hsl(config.brand.background),
  '--foreground': hsl(config.brand.foreground),
  '--muted': hsl(config.brand.muted),
  '--muted-foreground': hsl(config.brand.mutedForeground),
  '--card': hsl(config.brand.card),
  '--card-foreground': hsl(config.brand.cardForeground),
  '--border': hsl(config.brand.border),
  '--input': hsl(config.brand.input),
  '--ring': hsl(config.brand.ring),
  '--destructive': hsl(config.brand.destructive),
  '--destructive-foreground': hsl(config.brand.destructiveForeground),
};
```

**Dark Mode Injection (`.dark`):**
Same keys from `config.brandDark` applied to `.dark` selector.

**Implementation in App.tsx:**
```tsx
useEffect(() => {
  if (tenantConfig) {
    const root = document.documentElement;
    // Light
    Object.entries(tenantConfig.brand).forEach(([key, value]) => {
      root.style.setProperty(`--${kebabCase(key)}`, hexToHsl(value));
    });
    // Dark
    const darkSheet = getOrCreateDarkStyleSheet();
    Object.entries(tenantConfig.brandDark).forEach(([key, value]) => {
      darkSheet.insertRule(`.dark { --${kebabCase(key)}: ${hexToHsl(value)}; }`);
    });
  }
}, [tenantConfig]);
```

**Given** `tenantConfig.brand.primary = "#1A5F60"`  
**When** config loads  
**Then** `:root` has `--primary: 192 72% 21%`

**Given** dark mode active  
**When** `tenantConfig.brandDark.primary = "#2A8F90"`  
**Then** `.dark` has `--primary: 187 54% 36%`

---

### REQ-BI-002: Hex to HSL Conversion
All hex color strings SHALL be converted to HSL format `h s% l%` (space-separated, no commas) for CSS variable assignment.

```typescript
function hexToHsl(hex: string): string {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  let h = 0, s = 0, l = (max + min) / 2;
  
  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case r: h = (g - b) / d + (g < b ? 6 : 0); break;
      case g: h = (b - r) / d + 2; break;
      case b: h = (r - g) / d + 4; break;
    }
    h *= 60;
  }
  return `${Math.round(h)} ${Math.round(s * 100)}% ${Math.round(l * 100)}%`;
}
```

**Test Cases:**
| Input | Output |
|-------|--------|
| `#1A5F60` | `192 72% 21%` |
| `#E5F1EE` | `165 30% 92%` |
| `#E3645F` | `2 72% 63%` |
| `#2A8F90` | `187 54% 36%` |
| `#1E3A3A` | `180 33% 18%` |

---

### REQ-BI-003: Typography Variable Injection
The frontend SHALL inject typography tokens as CSS variables:

| Tenant Key | CSS Variable |
|------------|--------------|
| `typography.fontFamily` | `--font-family` |
| `typography.headingWeight` | `--heading-weight` |
| `typography.bodyWeight` | `--body-weight` |

**CSS Application (index.css):**
```css
@layer base {
  :root {
    --font-family: var(--font-family, 'Inter', system-ui, sans-serif);
    --heading-weight: var(--heading-weight, 700);
    --body-weight: var(--body-weight, 400);
  }
  
  h1, h2, h3, h4, h5, h6 {
    font-family: var(--font-family);
    font-weight: var(--heading-weight);
  }
  
  body, p, span, button, input, label, select, textarea {
    font-family: var(--font-family);
    font-weight: var(--body-weight);
  }
}
```

**Given** `typography.fontFamily = "'Inter', system-ui, sans-serif"`  
**When** config loads  
**Then** all text uses Inter font

---

### REQ-BI-004: Window Title from Commercial Name
The Tauri window title SHALL be set dynamically from `tenant.commercialName` after config loads.

```tsx
useEffect(() => {
  if (tenantConfig) {
    import('@tauri-apps/api/window')
      .then(({ getCurrentWindow }) => getCurrentWindow().setTitle(tenantConfig.tenant.commercialName))
      .catch(console.error);
  }
}, [tenantConfig]);
```

**Given** `commercialName = "MindLedger - Psic. Gloria Once"`  
**When** config loads  
**Then** window title bar shows "MindLedger - Psic. Gloria Once"

---

### REQ-BI-005: Login Page Auto-Theming
The Login page SHALL automatically inherit tenant colors via CSS variables — no component changes required.

**Verification:** Login page uses `bg-background`, `text-foreground`, `bg-primary`, `border-input`, etc. — all semantic Tailwind classes that resolve to CSS variables.

**Given** tenant config with brand colors  
**When** Login page renders  
**Then** form fields use tenant `--input`, `--border`  
**And** submit button uses tenant `--primary`, `--primary-foreground`  
**And** background uses tenant `--background`

---

### REQ-BI-006: Skeleton During Config Load
The app SHALL show a skeleton/loading state while tenant config fetches, avoiding flash of default colors.

```tsx
const { data: tenantConfig, isLoading } = useTenantConfig();

if (isLoading) {
  return <AppSkeleton />; // Uses neutral grays
}
```

**Given** app mounts, config not yet loaded  
**When** rendering  
**Then** shows skeleton with neutral styling

---

### REQ-BI-007: Tailwind Config Compatibility
The existing `tailwind.config.js` already references CSS variables for all semantic colors — this requirement confirms no Tailwind config changes needed.

**Current tailwind.config.js (verified):**
```js
colors: {
  primary: {
    DEFAULT: "hsl(var(--primary))",
    foreground: "hsl(var(--primary-foreground))",
  },
  secondary: {
    DEFAULT: "hsl(var(--secondary))",
    foreground: "hsl(var(--secondary-foreground))",
  },
  // ... all semantic colors use hsl(var(--*))
}
```

**Given** CSS variables injected per REQ-BI-001  
**When** Tailwind classes used  
**Then** components render with tenant colors automatically

---

## Scenarios

| Scenario | Given | When | Then |
|----------|-------|------|------|
| Full light theming | Gloria Once config | App loads | All components use Teal/Sage/Coral palette |
| Dark mode | Dark mode enabled | App loads | Components use brandDark palette |
| Typography | Custom font in config | App loads | All text uses tenant font |
| Window title | Config has commercialName | Config loads | Title bar updates |
| Login theming | Config has brand | Login renders | Form uses tenant colors |
| Fallback during load | Config fetching | Initial render | Skeleton shows, no color flash |
| Default build | No tenant config | App loads | Uses hardcoded CSS vars from index.css |