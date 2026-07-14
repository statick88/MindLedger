import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ToastProvider } from '@/hooks/use-toast';
import { Toaster } from '@/components/ui/Toaster';
import { Layout } from '@/components/layout/Layout';
import { DashboardPage } from '@/pages/DashboardPage';
import { PatientsPage } from '@/pages/PatientsPage';
import { AppointmentsPage } from '@/pages/AppointmentsPage';
import { ClinicalNotesPage } from '@/pages/ClinicalNotesPage';
import { AccountingPage } from '@/pages/AccountingPage';
import { SettingsPage } from '@/pages/SettingsPage';
import { useTenantConfig } from '@/hooks/useTenantConfig';
import { injectCssVariables } from '@/lib/color-utils';
import { useEffect } from 'react';
import './index.css';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5,
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

function AppContent() {
  const { data: tenantConfig, isLoading } = useTenantConfig();

  // Inject CSS variables when tenant config loads
  useEffect(() => {
    if (!tenantConfig) return;

    const { brand, brandDark, typography, tenant } = tenantConfig;

    // Inject all variables at once
    injectCssVariables(document.documentElement, brand, brandDark, typography);

    // Window title
    document.title = tenant.commercialName;
  }, [tenantConfig]);

  if (isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background">
        <div className="flex flex-col items-center gap-4">
          <div className="h-12 w-12 animate-spin rounded-full border-4 border-primary border-t-transparent" />
          <p className="text-muted-foreground">Cargando configuración...</p>
        </div>
      </div>
    );
  }

  return (
    <BrowserRouter>
      <ToastProvider>
        <Toaster />
        <Routes>
          <Route element={<Layout tenantConfig={tenantConfig} />}>
            <Route path="/" element={<DashboardPage />} />
            <Route path="/patients" element={<PatientsPage />} />
            <Route path="/appointments" element={<AppointmentsPage />} />
            <Route path="/clinical-notes" element={<ClinicalNotesPage />} />
            <Route path="/accounting" element={<AccountingPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Route>
        </Routes>
      </ToastProvider>
    </BrowserRouter>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AppContent />
    </QueryClientProvider>
  );
}

export default App;