import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { TenantConfig } from '@/types/tenant';

/**
 * Hook to fetch and cache tenant configuration.
 * Uses TanStack Query with infinite staleTime since config never changes at runtime.
 */
export function useTenantConfig() {
  return useQuery({
    queryKey: ['tenant-config'],
    queryFn: () => invoke<TenantConfig>('get_tenant_config'),
    staleTime: Infinity,    // Config never changes at runtime
    gcTime: Infinity,       // Keep in cache forever
    retry: false,           // Fail fast if command unavailable
  });
}