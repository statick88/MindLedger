'use client';

import { useQuery } from '@tanstack/react-query';
import { accountingApi } from '@/api';
import type { BalanceGeneral, EstadoResultados } from '@/api';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import { Skeleton } from '@/components/ui/Skeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { translateError } from '@/utils/translate-error';
import {
  Calculator,
  TrendingUp,
  TrendingDown,
  DollarSign,
  ArrowUpRight,
  ArrowDownRight,
  Plus,
  BookOpen,
} from 'lucide-react';

// ─── Format Currency ────────────────────────────────────────────────────────

function formatCurrency(amount: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
  }).format(amount);
}

// ─── Balance General Table ──────────────────────────────────────────────────

function BalanceGeneralView() {
  const { data, isLoading, error } = useQuery<BalanceGeneral>({
    queryKey: ['accounting', 'balance-general'],
    queryFn: () => accountingApi.balanceGeneral(),
  });

  if (isLoading) {
    return (
      <div className="space-y-4">
        <div className="grid gap-4 md:grid-cols-3">
          {[1, 2, 3].map((i) => (
            <Card key={i}>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <Skeleton className="h-4 w-16" />
                <Skeleton className="h-4 w-4 rounded" />
              </CardHeader>
              <CardContent>
                <Skeleton className="h-8 w-28" />
              </CardContent>
            </Card>
          ))}
        </div>
        <div className="grid gap-4 md:grid-cols-3">
          {[1, 2, 3].map((i) => (
            <Card key={i}>
              <CardHeader>
                <Skeleton className="h-5 w-16" />
              </CardHeader>
              <CardContent className="space-y-2">
                {[1, 2, 3].map((j) => (
                  <div key={j} className="flex justify-between">
                    <Skeleton className="h-3 w-24" />
                    <Skeleton className="h-3 w-20" />
                  </div>
                ))}
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <Card>
        <CardContent className="p-6 text-center">
          <p className="text-destructive">{translateError(error)}</p>
        </CardContent>
      </Card>
    );
  }

  if (!data) return null;

  return (
    <div className="space-y-4">
      {/* Summary Cards */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Activos</CardTitle>
            <TrendingUp className="h-4 w-4 text-teal-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatCurrency(data.total_activos)}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Pasivos</CardTitle>
            <TrendingDown className="h-4 w-4 text-coral" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatCurrency(data.total_pasivos)}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Patrimonio</CardTitle>
            <DollarSign className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatCurrency(data.total_patrimonio)}</div>
          </CardContent>
        </Card>
      </div>

      {/* Detailed Tables */}
      <div className="grid gap-4 md:grid-cols-3">
        {/* Activos */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Activos</CardTitle>
          </CardHeader>
          <CardContent>
            {data.activos.length === 0 ? (
              <p className="text-sm text-muted-foreground">Sin registros</p>
            ) : (
              <div className="space-y-2">
                {data.activos.map((item, i) => (
                  <div key={i} className="flex justify-between text-sm">
                    <span className="text-muted-foreground">{item.cuenta}</span>
                    <span className="font-medium">{formatCurrency(item.monto)}</span>
                  </div>
                ))}
                <div className="border-t pt-2 flex justify-between font-semibold">
                  <span>Total Activos</span>
                  <span>{formatCurrency(data.total_activos)}</span>
                </div>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Pasivos */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Pasivos</CardTitle>
          </CardHeader>
          <CardContent>
            {data.pasivos.length === 0 ? (
              <p className="text-sm text-muted-foreground">Sin registros</p>
            ) : (
              <div className="space-y-2">
                {data.pasivos.map((item, i) => (
                  <div key={i} className="flex justify-between text-sm">
                    <span className="text-muted-foreground">{item.cuenta}</span>
                    <span className="font-medium">{formatCurrency(item.monto)}</span>
                  </div>
                ))}
                <div className="border-t pt-2 flex justify-between font-semibold">
                  <span>Total Pasivos</span>
                  <span>{formatCurrency(data.total_pasivos)}</span>
                </div>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Patrimonio */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Patrimonio</CardTitle>
          </CardHeader>
          <CardContent>
            {data.patrimonio.length === 0 ? (
              <p className="text-sm text-muted-foreground">Sin registros</p>
            ) : (
              <div className="space-y-2">
                {data.patrimonio.map((item, i) => (
                  <div key={i} className="flex justify-between text-sm">
                    <span className="text-muted-foreground">{item.cuenta}</span>
                    <span className="font-medium">{formatCurrency(item.monto)}</span>
                  </div>
                ))}
                <div className="border-t pt-2 flex justify-between font-semibold">
                  <span>Total Patrimonio</span>
                  <span>{formatCurrency(data.total_patrimonio)}</span>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Accounting Equation */}
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center justify-center gap-4 text-lg">
            <span className="font-medium">Activos</span>
            <span className="text-muted-foreground">=</span>
            <span className="font-medium">Pasivos</span>
            <span className="text-muted-foreground">+</span>
            <span className="font-medium">Patrimonio</span>
          </div>
          <div className="flex items-center justify-center gap-4 text-sm text-muted-foreground mt-1">
            <span>{formatCurrency(data.total_activos)}</span>
            <span>=</span>
            <span>{formatCurrency(data.total_pasivos)}</span>
            <span>+</span>
            <span>{formatCurrency(data.total_patrimonio)}</span>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// ─── Estado de Resultados Table ─────────────────────────────────────────────

function EstadoResultadosView() {
  const { data, isLoading, error } = useQuery<EstadoResultados>({
    queryKey: ['accounting', 'estado-resultados'],
    queryFn: () => accountingApi.estadoResultados(),
  });

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Card>
          <CardContent className="p-6">
            <div className="text-center space-y-2">
              <Skeleton className="h-4 w-24 mx-auto" />
              <Skeleton className="h-10 w-40 mx-auto" />
              <Skeleton className="h-5 w-16 mx-auto rounded-full" />
            </div>
          </CardContent>
        </Card>
        <div className="grid gap-4 md:grid-cols-2">
          {[1, 2].map((i) => (
            <Card key={i}>
              <CardHeader>
                <Skeleton className="h-5 w-20" />
              </CardHeader>
              <CardContent className="space-y-2">
                {[1, 2, 3].map((j) => (
                  <div key={j} className="flex justify-between">
                    <Skeleton className="h-3 w-28" />
                    <Skeleton className="h-3 w-20" />
                  </div>
                ))}
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <Card>
        <CardContent className="p-6 text-center">
          <p className="text-destructive">{translateError(error)}</p>
        </CardContent>
      </Card>
    );
  }

  if (!data) return null;

  const isProfit = data.utilidad_neta >= 0;

  return (
    <div className="space-y-4">
      {/* Summary */}
      <Card>
        <CardContent className="p-6">
          <div className="text-center">
            <p className="text-sm text-muted-foreground mb-1">Utilidad Neta</p>
            <p className={`text-4xl font-bold ${isProfit ? 'text-teal-500' : 'text-coral'}`}>
              {formatCurrency(data.utilidad_neta)}
            </p>
            <Badge variant={isProfit ? 'default' : 'destructive'} className="mt-2">
              {isProfit ? 'Ganancia' : 'Pérdida'}
            </Badge>
          </div>
        </CardContent>
      </Card>

      <div className="grid gap-4 md:grid-cols-2">
        {/* Ingresos */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2">
              <ArrowUpRight className="h-5 w-5 text-teal-500" />
              Ingresos
            </CardTitle>
          </CardHeader>
          <CardContent>
            {data.ingresos.length === 0 ? (
              <p className="text-sm text-muted-foreground">Sin registros</p>
            ) : (
              <div className="space-y-2">
                {data.ingresos.map((item, i) => (
                  <div key={i} className="flex justify-between text-sm">
                    <span className="text-muted-foreground">{item.cuenta}</span>
                    <span className="font-medium text-teal-500">{formatCurrency(item.monto)}</span>
                  </div>
                ))}
                <div className="border-t pt-2 flex justify-between font-semibold">
                  <span>Total Ingresos</span>
                  <span className="text-teal-500">{formatCurrency(data.total_ingresos)}</span>
                </div>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Gastos */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2">
              <ArrowDownRight className="h-5 w-5 text-coral" />
              Gastos
            </CardTitle>
          </CardHeader>
          <CardContent>
            {data.gastos.length === 0 ? (
              <p className="text-sm text-muted-foreground">Sin registros</p>
            ) : (
              <div className="space-y-2">
                {data.gastos.map((item, i) => (
                  <div key={i} className="flex justify-between text-sm">
                    <span className="text-muted-foreground">{item.cuenta}</span>
                    <span className="font-medium text-coral">{formatCurrency(item.monto)}</span>
                  </div>
                ))}
                <div className="border-t pt-2 flex justify-between font-semibold">
                  <span>Total Gastos</span>
                  <span className="text-coral">{formatCurrency(data.total_gastos)}</span>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

// ─── Asientos List ──────────────────────────────────────────────────────────

function AsientosList() {
  const { data, isLoading } = useQuery({
    queryKey: ['accounting', 'asientos'],
    queryFn: () => accountingApi.listAsientos({ page: 1, page_size: 50 }),
  });

  const asientos = data?.items ?? [];

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
          <CardTitle>Asientos Contables</CardTitle>
          <CardDescription>Registro de movimientos</CardDescription>
        </div>
        <Button size="sm" className="gap-2">
          <Plus className="h-4 w-4" />
          Nuevo Asiento
        </Button>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="space-y-2">
            {[1, 2, 3, 4, 5].map((i) => (
              <div key={i} className="flex items-center gap-4 py-2">
                <Skeleton className="h-3 w-20" />
                <Skeleton className="h-3 w-48" />
                <Skeleton className="h-5 w-16 rounded-full" />
                <Skeleton className="h-3 w-20 ml-auto" />
                <Skeleton className="h-3 w-20" />
              </div>
            ))}
          </div>
        ) : asientos.length === 0 ? (
          <EmptyState
            icon={<BookOpen className="h-8 w-8" />}
            title="Sin asientos contables"
            description="Registra tu primer asiento para llevar el control financiero de tu práctica."
            action={{ label: 'Nuevo Asiento', onClick: () => {} }}
          />
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b">
                  <th className="text-left py-2 font-medium text-muted-foreground">Fecha</th>
                  <th className="text-left py-2 font-medium text-muted-foreground">Descripción</th>
                  <th className="text-left py-2 font-medium text-muted-foreground">Categoría</th>
                  <th className="text-right py-2 font-medium text-muted-foreground">Debe</th>
                  <th className="text-right py-2 font-medium text-muted-foreground">Haber</th>
                </tr>
              </thead>
              <tbody>
                {asientos.map((asiento) => (
                  <tr key={asiento.id} className="border-b last:border-0">
                    <td className="py-2">{asiento.fecha}</td>
                    <td className="py-2">{asiento.descripcion}</td>
                    <td className="py-2">
                      <Badge variant="outline">{asiento.categoria}</Badge>
                    </td>
                    <td className="py-2 text-right font-medium">
                      {asiento.debe > 0 ? formatCurrency(asiento.debe) : '—'}
                    </td>
                    <td className="py-2 text-right font-medium">
                      {asiento.haber > 0 ? formatCurrency(asiento.haber) : '—'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// ─── Main Accounting Page ───────────────────────────────────────────────────

export function AccountingPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-primary">Contabilidad</h1>
        <p className="text-muted-foreground">Estado financiero de la práctica</p>
      </div>

      <Tabs defaultValue="balance" className="space-y-4">
        <TabsList>
          <TabsTrigger value="balance" className="gap-2">
            <Calculator className="h-4 w-4" />
            Balance General
          </TabsTrigger>
          <TabsTrigger value="resultados" className="gap-2">
            <TrendingUp className="h-4 w-4" />
            Estado de Resultados
          </TabsTrigger>
          <TabsTrigger value="asientos" className="gap-2">
            <DollarSign className="h-4 w-4" />
            Asientos
          </TabsTrigger>
        </TabsList>

        <TabsContent value="balance">
          <BalanceGeneralView />
        </TabsContent>

        <TabsContent value="resultados">
          <EstadoResultadosView />
        </TabsContent>

        <TabsContent value="asientos">
          <AsientosList />
        </TabsContent>
      </Tabs>
    </div>
  );
}
