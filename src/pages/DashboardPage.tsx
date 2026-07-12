'use client';

import { LayoutDashboard } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';

const stats = [
  { name: 'Pacientes Totales', value: '1,234', change: '+12%', icon: LayoutDashboard, color: 'text-blue-500' },
  { name: 'Turnos Hoy', value: '24', change: '+3', icon: LayoutDashboard, color: 'text-green-500' },
  { name: 'Historias Clínicas', value: '567', change: '+8%', icon: LayoutDashboard, color: 'text-purple-500' },
  { name: 'Tasa Ocupación', value: '78%', change: '+5%', icon: LayoutDashboard, color: 'text-orange-500' },
];

export function DashboardPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground">Visión general del sistema</p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat, index) => (
          <Card key={index}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">{stat.name}</CardTitle>
              <stat.icon className={`h-4 w-4 ${stat.color}`} />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stat.value}</div>
              <p className="text-xs text-muted-foreground">{stat.change} vs mes anterior</p>
            </CardContent>
          </Card>
        ))}
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Acciones Rápidas</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <Button className="w-full justify-start">Nuevo Paciente</Button>
            <Button variant="outline" className="w-full justify-start">Nuevo Turno</Button>
            <Button variant="outline" className="w-full justify-start">Nueva Historia Clínica</Button>
            <Button variant="outline" className="w-full justify-start">Ver Agenda</Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Próximos Turnos</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <div className="flex items-center justify-between p-3 bg-accent rounded-md">
                <div>
                  <p className="font-medium">Juan Pérez</p>
                  <p className="text-sm text-muted-foreground">Consulta General • 10:00 AM</p>
                </div>
                <span className="text-sm text-green-600">Confirmado</span>
              </div>
              <div className="flex items-center justify-between p-3 bg-accent rounded-md">
                <div>
                  <p className="font-medium">María González</p>
                  <p className="text-sm text-muted-foreground">Control • 11:30 AM</p>
                </div>
                <span className="text-sm text-blue-600">Pendiente</span>
              </div>
              <div className="flex items-center justify-between p-3 bg-accent rounded-md">
                <div>
                  <p className="font-medium">Carlos Rodríguez</p>
                  <p className="text-sm text-muted-foreground">Urgencia • 02:00 PM</p>
                </div>
                <span className="text-sm text-red-600">En Espera</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}