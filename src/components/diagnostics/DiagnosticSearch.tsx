'use client';

import { useState, useCallback } from 'react';
import { useQuery } from '@tanstack/react-query';
import { diagnosticsApi } from '@/api';
import type { Cie10Entry, Dsm5Entry } from '@/api';
import { Input } from '@/components/ui/Input';
import { Badge } from '@/components/ui/Badge';
import { Card, CardContent } from '@/components/ui/Card';
import { Search, Loader2, X } from 'lucide-react';

interface DiagnosticSearchProps {
  /** Called when user selects a diagnostic entry */
  onSelect: (entry: { code: string; description: string; system: 'CIE-10' | 'DSM-5' }) => void;
  /** Placeholder text */
  placeholder?: string;
  /** Which system to search: 'CIE-10', 'DSM-5', or 'both' */
  system?: 'CIE-10' | 'DSM-5' | 'both';
  /** Currently selected value (controlled) */
  value?: string;
  /** Called when selection is cleared */
  onClear?: () => void;
}

/**
 * Diagnostic search Combobox — searches CIE-10 and/or DSM-5 via IPC.
 *
 * Usage:
 * ```tsx
 * <DiagnosticSearch
 *   onSelect={(entry) => console.log(entry)}
 *   system="both"
 * />
 * ```
 */
export function DiagnosticSearch({
  onSelect,
  placeholder = 'Buscar diagnóstico (CIE-10 / DSM-5)...',
  system = 'both',
  value,
  onClear,
}: DiagnosticSearchProps) {
  const [query, setQuery] = useState(value ?? '');
  const [isOpen, setIsOpen] = useState(false);

  const { data: cie10Results, isLoading: cie10Loading } = useQuery<Cie10Entry[]>({
    queryKey: ['cie10', query],
    queryFn: () => diagnosticsApi.searchCie10(query, 10),
    enabled: query.length >= 2 && (system === 'CIE-10' || system === 'both'),
    staleTime: 1000 * 60 * 5,
  });

  const { data: dsm5Results, isLoading: dsm5Loading } = useQuery<Dsm5Entry[]>({
    queryKey: ['dsm5', query],
    queryFn: () => diagnosticsApi.searchDsm5(query, 10),
    enabled: query.length >= 2 && (system === 'DSM-5' || system === 'both'),
    staleTime: 1000 * 60 * 5,
  });

  const isLoading = cie10Loading || dsm5Loading;
  const hasResults = (cie10Results?.length ?? 0) > 0 || (dsm5Results?.length ?? 0) > 0;

  const handleSelect = useCallback(
    (entry: { code: string; description: string; system: 'CIE-10' | 'DSM-5' }) => {
      setQuery(`${entry.code} — ${entry.description}`);
      setIsOpen(false);
      onSelect(entry);
    },
    [onSelect]
  );

  const handleClear = () => {
    setQuery('');
    onClear?.();
  };

  return (
    <div className="relative">
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setIsOpen(true);
          }}
          onFocus={() => setIsOpen(true)}
          placeholder={placeholder}
          className="pl-9 pr-8"
        />
        {query && (
          <button
            onClick={handleClear}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
          >
            <X className="h-4 w-4" />
          </button>
        )}
        {isLoading && (
          <div className="absolute right-8 top-1/2 -translate-y-1/2">
            <Loader2 className="h-4 w-4 animate-spin text-primary" />
          </div>
        )}
      </div>

      {/* Results Dropdown */}
      {isOpen && query.length >= 2 && (
        <Card className="absolute z-50 w-full mt-1 max-h-80 overflow-y-auto shadow-lg">
          <CardContent className="p-0">
            {!isLoading && !hasResults && (
              <div className="p-4 text-center text-sm text-muted-foreground">
                No se encontraron resultados para "{query}"
              </div>
            )}

            {/* CIE-10 Results */}
            {cie10Results && cie10Results.length > 0 && (
              <div>
                <div className="px-3 py-1.5 text-xs font-semibold text-muted-foreground bg-muted/50">
                  CIE-10
                </div>
                {cie10Results.map((entry) => (
                  <button
                    key={entry.id}
                    onClick={() =>
                      handleSelect({
                        code: entry.codigo,
                        description: entry.descripcion,
                        system: 'CIE-10',
                      })
                    }
                    className="w-full text-left px-3 py-2 hover:bg-accent transition-colors flex items-start gap-2"
                  >
                    <Badge variant="outline" className="mt-0.5 flex-shrink-0 text-xs">
                      {entry.codigo}
                    </Badge>
                    <div className="min-w-0">
                      <p className="text-sm truncate">{entry.descripcion}</p>
                      <p className="text-xs text-muted-foreground">{entry.categoria}</p>
                    </div>
                  </button>
                ))}
              </div>
            )}

            {/* DSM-5 Results */}
            {dsm5Results && dsm5Results.length > 0 && (
              <div>
                <div className="px-3 py-1.5 text-xs font-semibold text-muted-foreground bg-muted/50">
                  DSM-5
                </div>
                {dsm5Results.map((entry) => (
                  <button
                    key={entry.id}
                    onClick={() =>
                      handleSelect({
                        code: entry.codigo,
                        description: entry.descripcion,
                        system: 'DSM-5',
                      })
                    }
                    className="w-full text-left px-3 py-2 hover:bg-accent transition-colors flex items-start gap-2"
                  >
                    <Badge variant="secondary" className="mt-0.5 flex-shrink-0 text-xs">
                      {entry.codigo}
                    </Badge>
                    <div className="min-w-0">
                      <p className="text-sm truncate">{entry.descripcion}</p>
                      <p className="text-xs text-muted-foreground">{entry.categoria}</p>
                    </div>
                  </button>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Click outside to close */}
      {isOpen && (
        <div className="fixed inset-0 z-40" onClick={() => setIsOpen(false)} />
      )}
    </div>
  );
}
