# Frontend Layout — Delta Spec

## MODIFIED Requirements

### REQ-LAYOUT-001: Sidebar Brand Dynamic (MODIFIED)
**Previously:** Layout.tsx hardcoded the brand:
```tsx
<div className="flex items-center gap-2">
  <div className="h-8 w-8 rounded-lg bg-primary flex items-center justify-center">
    <span className="text-primary-foreground font-bold text-sm">M</span>
  </div>
  <span className="text-xl font-bold text-primary">MindLedger</span>
</div>
```

**Updated:** Layout.tsx reads brand from `useTenantConfig` hook:
```tsx
import { useTenantConfig } from '@/hooks/useTenantConfig';

export function Layout() {
  const { data: tenantConfig, isLoading } = useTenantConfig();
  
  // ... existing code ...
  
  return (
    <aside className="...">
      <div className="flex h-16 items-center justify-between border-b px-4">
        {!sidebarCollapsed && tenantConfig && (
          <div className="flex items-center gap-2">
            <div className="h-8 w-8 rounded-lg bg-primary flex items-center justify-center">
              <span className="text-primary-foreground font-bold text-sm">
                {tenantConfig.tenant.commercialName.charAt(0)}
              </span>
            </div>
            <div className="flex flex-col min-w-0">
              <span className="text-xl font-bold text-primary truncate">
                {tenantConfig.tenant.commercialName}
              </span>
              <span className="text-xs text-muted-foreground truncate">
                {tenantConfig.tenant.clinicalRole}
              </span>
            </div>
          </div>
        )}
        {!sidebarCollapsed && !tenantConfig && (
          // Fallback during loading
          <div className="flex items-center gap-2">
            <div className="h-8 w-8 rounded-lg bg-primary flex items-center justify-center">
              <span className="text-primary-foreground font-bold text-sm">M</span>
            </div>
            <span className="text-xl font-bold text-primary">MindLedger</span>
          </div>
        )}
        {/* ... existing buttons ... */}
      </div>
      {/* ... rest of sidebar ... */}
    </aside>
  );
}
```

(Previously: hardcoded "M" + "MindLedger")

#### Scenario: Tenant config loaded
- GIVEN `useTenantConfig` returns Gloria Once config
- WHEN sidebar renders (expanded)
- THEN shows "M" logo + "MindLedger - Psic. Gloria Once" + "Neuropsicóloga Clínica" subtitle

#### Scenario: Config loading (fallback)
- GIVEN `isLoading = true`
- WHEN sidebar renders
- THEN shows hardcoded "MindLedger" fallback

#### Scenario: Collapsed sidebar
- GIVEN sidebarCollapsed = true
- WHEN sidebar renders
- THEN shows only logo (first char of commercialName)

---

### REQ-LAYOUT-002: Window Title Runtime Override (MODIFIED)
**Previously:** Window title came only from `tauri.conf.json` hardcoded value.

**Updated:** `App.tsx` sets window title dynamically from tenant config on mount/update:

```tsx
import { useTenantConfig } from '@/hooks/useTenantConfig';
import { useEffect } from 'react';

function App() {
  const { data: tenantConfig } = useTenantConfig();
  
  useEffect(() => {
    if (tenantConfig) {
      import('@tauri-apps/api/window')
        .then(({ getCurrentWindow }) => {
          getCurrentWindow().setTitle(tenantConfig.tenant.commercialName);
        })
        .catch(console.error);
    }
  }, [tenantConfig]);
  
  // ... rest of App
}
```

(Previously: static title from tauri.conf.json only)

#### Scenario: Config loads after app start
- GIVEN app starts, config fetching
- WHEN `useTenantConfig` resolves
- THEN window title updates to `commercialName`

---

## ADDED Requirements

### REQ-FL-001: useTenantConfig Hook (ADDED)
**File:** `src/hooks/useTenantConfig.ts`

```typescript
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';

export interface TenantConfig {
  tenant: {
    id: string;
    commercialName: string;
    clinicalRole: string;
    ownerName: string;
    ownerTitle: string;
  };
  brand: Record<string, string>;
  brandDark: Record<string, string>;
  typography: {
    fontFamily: string;
    headingWeight: string;
    bodyWeight: string;
  };
  crypto: {
    keyringService: string;
    keyringAccount: string;
    dbFileName: string;
  };
  features: {
    clinicalNotes: boolean;
    accounting: boolean;
    agenda: boolean;
    diagnostics: boolean;
  };
}

export function useTenantConfig() {
  return useQuery({
    queryKey: ['tenantConfig'],
    queryFn: () => invoke<TenantConfig>('get_tenant_config'),
    staleTime: Infinity,
    retry: false,
  });
}
```

---

## REMOVED Requirements

*(None)*

---

## Scenarios

| Scenario | Given | When | Then |
|----------|-------|------|------|
| Brand from config | Config loaded | Layout renders | commercialName + clinicalRole shown |
| Fallback loading | Config fetching | Layout renders | Hardcoded "MindLedger" shown |
| Collapsed sidebar | Config loaded, collapsed | Sidebar renders | Only logo (first char) shown |
| Window title | Config loaded | App mounted | Tauri window title = commercialName |
| Hook caching | Component remounts | useTenantConfig called | No re-fetch, cached data returned |