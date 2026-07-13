'use client';

import { useQuery } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { patientApi, appointmentApi, clinicalNoteApi, accountingApi } from '@/api';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import {
  Users,
  Calendar,
  FileText,
  DollarSign,
  ArrowUpRight,
  AlertCircle,
} from 'lucide-react';
import { Skeleton } from '@/components/ui/Skeleton';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
} from 'recharts';


// ─── Helpers ────────────────────────────────────────────────────────────────

function formatCurrency(amount: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 0,
  }).format(amount);
}

function getToday(): string {
  return new Date().toISOString().split('T')[0];
}

const STATUS_COLORS: Record<string, string> = {
  Scheduled: '#0F4C5C',
  Confirmed: '#22C55E',
  InProgress: '#EAB308',
  Completed: '#6B7280',
  Cancelled: '#E3645F',
  NoShow: '#DC2626',
};

const STATUS_LABELS: Record<string, string> = {
  Scheduled: 'Programada',
  Confirmed: 'Confirmada',
  InProgress: 'En Progreso',
  Completed: 'Completada',
  Cancelled: 'Cancelada',
  NoShow: 'No Asistió',
};

// ─── Stat Card ──────────────────────────────────────────────────────────────

function StatCard({
  title,
  value,
  subtitle,
  icon: Icon,
  color,
  loading,
}: {
  title: string;
  value: string;
  subtitle?: string;
  icon: typeof Users;
  color: string;
  loading?: boolean;
}) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <Icon className={`h-4 w-4 ${color}`} />
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="space-y-2">
            <Skeleton className="h-8 w-24" />
            <Skeleton className="h-3 w-16" />
          </div>
        ) : (
          <>
            <div className="text-2xl font-bold">{value}</div>
            {subtitle && <p className="text-xs text-muted-foreground">{subtitle}</p>}
          </>
        )}
      </CardContent>
    </Card>
  );
}

// ─── Main Dashboard ─────────────────────────────────────────────────────────

export function DashboardPage() {
  const navigate = useNavigate();
  const today = getToday();

  // ── Queries ──

  const { data: totalPatients, isLoading: patientsLoading } = useQuery({
    queryKey: ['patients', 'count'],
    queryFn: () => patientApi.count(true),
  });

  const { data: todayAppointments, isLoading: appointmentsLoading } = useQuery({
    queryKey: ['appointments', 'today', today],
    queryFn: () => appointmentApi.byDate(today, 1, 50),
  });

  const { data: unsignedNotes, isLoading: notesLoading } = useQuery({
    queryKey: ['clinical-notes', 'unsigned'],
    queryFn: () => clinicalNoteApi.unsigned(1, 100),
  });

  const { data: balanceGeneral, isLoading: balanceLoading } = useQuery({
    queryKey: ['accounting', 'balance-general'],
    queryFn: () => accountingApi.balanceGeneral(),
  });

  // ── Derived Data ──

  const appointments = todayAppointments?.items ?? [];
  const unsignedCount = unsignedNotes?.total ?? 0;

  // Appointments by status (for bar chart)
  const statusCounts: Record<string, number> = {};
  appointments.forEach((apt) => {
    statusCounts[apt.status] = (statusCounts[apt.status] || 0) + 1;
  });
  const statusData = Object.entries(statusCounts).map(([status, count]) => ({
    name: STATUS_LABELS[status] || status,
    value: count,
    fill: STATUS_COLORS[status] || '#6B7280',
  }));

  // Appointment type distribution (for pie chart)
  const typeCounts: Record<string, number> = {};
  appointments.forEach((apt) => {
    typeCounts[apt.appointment_type] = (typeCounts[apt.appointment_type] || 0) + 1;
  });
  const typeData = Object.entries(typeCounts).map(([type, count]) => ({
    name: type,
    value: count,
  }));

  const PIE_COLORS = ['#0F4C5C', '#E5F1EE', '#E3645F', '#22C55E', '#EAB308', '#8B5CF6'];

  // ── Upcoming (first 5) ──

  const upcoming = appointments
    .filter((a) => a.status === 'Scheduled' || a.status === 'Confirmed')
    .sort((a, b) => a.scheduled_time.localeCompare(b.scheduled_time))
    .slice(0, 5);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-primary">Dashboard</h1>
        <p className="text-muted-foreground">
          {new Date().toLocaleDateString('es-EC', {
            weekday: 'long',
            year: 'numeric',
            month: 'long',
            day: 'numeric',
          })}
        </p>
      </div>

      {/* Stat Cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <StatCard
          title="Pacientes Activos"
          value={totalPatients?.toString() ?? '—'}
          icon={Users}
          color="text-teal-500"
          loading={patientsLoading}
        />
        <StatCard
          title="Citas Hoy"
          value={appointments.length.toString()}
          subtitle={`${appointments.filter((a) => a.status === 'Completed').length} completadas`}
          icon={Calendar}
          color="text-sage"
          loading={appointmentsLoading}
        />
        <StatCard
          title="Notas sin Firmar"
          value={unsignedCount.toString()}
          icon={FileText}
          color="text-coral"
          loading={notesLoading}
        />
        <StatCard
          title="Activos Totales"
          value={balanceGeneral ? formatCurrency(balanceGeneral.total_activos) : '—'}
          icon={DollarSign}
          color="text-teal-500"
          loading={balanceLoading}
        />
      </div>

      {/* Charts Row */}
      {appointmentsLoading ? (
        <div className="grid gap-4 md:grid-cols-2">
          <Card>
            <CardHeader>
              <Skeleton className="h-5 w-32" />
            </CardHeader>
            <CardContent>
              <Skeleton className="h-[200px] w-full rounded-lg" />
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <Skeleton className="h-5 w-40" />
            </CardHeader>
            <CardContent>
              <Skeleton className="h-[200px] w-full rounded-lg" />
            </CardContent>
          </Card>
        </div>
      ) : appointments.length > 0 && (
        <div className="grid gap-4 md:grid-cols-2">
          {/* Appointments by Status */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Citas por Estado</CardTitle>
            </CardHeader>
            <CardContent>
              {statusData.length === 0 ? (
                <p className="text-sm text-muted-foreground text-center py-8">Sin citas hoy</p>
              ) : (
                <ResponsiveContainer width="100%" height={200}>
                  <BarChart data={statusData}>
                    <CartesianGrid strokeDasharray="3 3" className="opacity-30" />
                    <XAxis dataKey="name" tick={{ fontSize: 11 }} />
                    <YAxis allowDecimals={false} tick={{ fontSize: 11 }} />
                    <Tooltip />
                    <Bar dataKey="value" radius={[4, 4, 0, 0]}>
                      {statusData.map((entry, i) => (
                        <Cell key={i} fill={entry.fill} />
                      ))}
                    </Bar>
                  </BarChart>
                </ResponsiveContainer>
              )}
            </CardContent>
          </Card>

          {/* Appointment Type Distribution */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Distribución por Tipo</CardTitle>
            </CardHeader>
            <CardContent>
              {typeData.length === 0 ? (
                <p className="text-sm text-muted-foreground text-center py-8">Sin citas hoy</p>
              ) : (
                <ResponsiveContainer width="100%" height={200}>
                  <PieChart>
                    <Pie
                      data={typeData}
                      cx="50%"
                      cy="50%"
                      outerRadius={80}
                      dataKey="value"
                      label={({ name, percent }) =>
                        `${name ?? ''} (${((percent ?? 0) * 100).toFixed(0)}%)`
                      }
                      labelLine={false}
                    >
                      {typeData.map((_, i) => (
                        <Cell key={i} fill={PIE_COLORS[i % PIE_COLORS.length]} />
                      ))}
                    </Pie>
                    <Tooltip />
                  </PieChart>
                </ResponsiveContainer>
              )}
            </CardContent>
          </Card>
        </div>
      )}

      {/* Bottom Row */}
      <div className="grid gap-4 md:grid-cols-2">
        {/* Upcoming Appointments */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle className="text-lg">Próximas Citas</CardTitle>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => navigate('/appointments')}
              className="gap-1"
            >
              Ver agenda
              <ArrowUpRight className="h-3 w-3" />
            </Button>
          </CardHeader>
          <CardContent>
            {appointmentsLoading ? (
              <div className="space-y-3">
                {[1, 2, 3].map((i) => (
                  <div key={i} className="flex items-center justify-between p-3 bg-accent/50 rounded-lg">
                    <div className="space-y-2 flex-1">
                      <Skeleton className="h-4 w-32" />
                      <Skeleton className="h-3 w-48" />
                    </div>
                    <Skeleton className="h-5 w-16 rounded-full" />
                  </div>
                ))}
              </div>
            ) : upcoming.length === 0 ? (
              <p className="text-sm text-muted-foreground text-center py-8">
                No hay citas programadas
              </p>
            ) : (
              <div className="space-y-3">
                {upcoming.map((apt) => (
                  <div
                    key={apt.id}
                    className="flex items-center justify-between p-3 bg-accent/50 rounded-lg"
                  >
                    <div className="min-w-0">
                      <p className="font-medium truncate">{apt.reason}</p>
                      <p className="text-sm text-muted-foreground">
                        {apt.scheduled_time} · {apt.duration_minutes} min
                        {apt.room ? ` · ${apt.room}` : ''}
                      </p>
                    </div>
                    <Badge variant={apt.status === 'Confirmed' ? 'default' : 'secondary'}>
                      {STATUS_LABELS[apt.status] || apt.status}
                    </Badge>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Quick Actions */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Acciones Rápidas</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <Button className="w-full justify-start gap-2" onClick={() => navigate('/patients')}>
              <Users className="h-4 w-4" />
              Nuevo Paciente
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start gap-2"
              onClick={() => navigate('/appointments')}
            >
              <Calendar className="h-4 w-4" />
              Nueva Cita
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start gap-2"
              onClick={() => navigate('/accounting')}
            >
              <DollarSign className="h-4 w-4" />
              Ver Contabilidad
            </Button>
            {unsignedCount > 0 && (
              <div className="flex items-center gap-2 text-sm text-coral mt-2">
                <AlertCircle className="h-4 w-4" />
                <span>{unsignedCount} notas clínicas pendientes de firma</span>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
