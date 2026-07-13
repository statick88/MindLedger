'use client';

import { useState, useMemo, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { agendaApi, patientApi } from '@/api';
import type {
  AppointmentResponse,
  CreateAppointmentRequest,
  UpdateAppointmentRequest,
  Modality,
} from '@/types/agenda';
import { STATUS_COLORS, MODALITY_LABELS, STATUS_LABELS, formatFeeCents } from '@/types/agenda';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import { Textarea } from '@/components/ui/Textarea';
import { Skeleton } from '@/components/ui/Skeleton';
import { useToast } from '@/hooks/use-toast';
import {
  Plus,
  ChevronLeft,
  ChevronRight,
  CalendarDays,
  X,
  Search,
  Clock,
  DollarSign,
  TrendingUp,
  Loader2,
  CheckCircle2,
  Trash2,
} from 'lucide-react';
import {
  format,
  startOfWeek,
  endOfWeek,
  addWeeks,
  subWeeks,
  isSameDay,
  addDays,
  setHours,
  setMinutes,
  parseISO,
} from 'date-fns';
import { es } from 'date-fns/locale';

// ─── Constants ──────────────────────────────────────────────────────────────

const HOUR_START = 8;
const HOUR_END = 19;
const HOURS = Array.from({ length: HOUR_END - HOUR_START }, (_, i) => HOUR_START + i);

const MODALITIES: Modality[] = ['Presencial', 'Virtual', 'Hibrida'];

// ─── Helpers ────────────────────────────────────────────────────────────────

function toLocalDatetime(iso: string): string {
  const d = parseISO(iso);
  const pad = (n: number) => n.toString().padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function fromLocalDatetime(local: string): string {
  return new Date(local).toISOString();
}

function getMinutesFromStart(iso: string): number {
  const d = parseISO(iso);
  return d.getHours() * 60 + d.getMinutes();
}

function slotTop(iso: string): number {
  const mins = getMinutesFromStart(iso);
  return ((mins - HOUR_START * 60) / 60) * 64; // 64px per hour
}

function slotHeight(startIso: string, endIso: string): number {
  const startMins = getMinutesFromStart(startIso);
  const endMins = getMinutesFromStart(endIso);
  return Math.max(((endMins - startMins) / 60) * 64, 24);
}

// ─── Empty form state ──────────────────────────────────────────────────────

interface AppointmentForm {
  patient_id: string;
  patient_name: string;
  therapist_id: string;
  modality: Modality;
  start_at: string;
  end_at: string;
  fee_cents: number;
  notes: string;
  reminder_minutes_before: number | null;
}

const EMPTY_FORM: AppointmentForm = {
  patient_id: '',
  patient_name: '',
  therapist_id: '',
  modality: 'Presencial',
  start_at: '',
  end_at: '',
  fee_cents: 0,
  notes: '',
  reminder_minutes_before: 30,
};

// ─── Component ──────────────────────────────────────────────────────────────

export function AppointmentsPage() {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  const [currentWeek, setCurrentWeek] = useState<Date>(new Date());
  const [panelOpen, setPanelOpen] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [form, setForm] = useState<AppointmentForm>(EMPTY_FORM);
  const [patientQuery, setPatientQuery] = useState('');
  const [finalizedEntry, setFinalizedEntry] = useState<{ id: string; descripcion: string } | null>(null);

  const weekStart = startOfWeek(currentWeek, { weekStartsOn: 1 });
  const weekEnd = endOfWeek(currentWeek, { weekStartsOn: 1 });
  const days = Array.from({ length: 7 }, (_, i) => addDays(weekStart, i));

  // ─── Queries ────────────────────────────────────────────────────────────

  const { data: appointments = [], isLoading } = useQuery<AppointmentResponse[]>({
    queryKey: ['agenda', 'citas', weekStart.toISOString(), weekEnd.toISOString()],
    queryFn: () => agendaApi.citasRango(weekStart.toISOString(), weekEnd.toISOString()),
  });

  const { data: metrics } = useQuery({
    queryKey: ['agenda', 'metricas', weekStart.toISOString(), weekEnd.toISOString()],
    queryFn: () => agendaApi.metricas(weekStart.toISOString(), weekEnd.toISOString()),
  });

  const { data: patientResults = [] } = useQuery<any[]>({
    queryKey: ['patients', 'search', patientQuery],
    queryFn: () => patientApi.search(patientQuery, 1, 8).then((r) => r.items),
    enabled: patientQuery.length >= 2 && panelOpen,
    staleTime: 1000 * 60 * 5,
  });

  // ─── Mutations ──────────────────────────────────────────────────────────

  const createMutation = useMutation({
    mutationFn: (req: CreateAppointmentRequest) => agendaApi.crear(req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agenda'] });
      toast({ title: 'Cita creada', description: 'La cita se ha registrado correctamente.' });
      closePanel();
    },
    onError: (e: any) => {
      toast({ title: 'Error', description: e?.toString() ?? 'No se pudo crear la cita.', variant: 'destructive' });
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, req }: { id: string; req: UpdateAppointmentRequest }) =>
      agendaApi.actualizar(id, req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agenda'] });
      toast({ title: 'Cita actualizada', description: 'Los cambios se han guardado.' });
      closePanel();
    },
    onError: (e: any) => {
      toast({ title: 'Error', description: e?.toString() ?? 'No se pudo actualizar.', variant: 'destructive' });
    },
  });

  const cancelMutation = useMutation({
    mutationFn: (id: string) => agendaApi.cancelar(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agenda'] });
      toast({ title: 'Cita cancelada' });
      closePanel();
    },
    onError: (e: any) => {
      toast({ title: 'Error', description: e?.toString() ?? 'No se pudo cancelar.', variant: 'destructive' });
    },
  });

  const finalizeMutation = useMutation({
    mutationFn: (id: string) => agendaApi.finalizarSesion(id),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['agenda'] });
      setFinalizedEntry({ id: data.id, descripcion: `Sesión finalizada — ${formatFeeCents(data.fee_cents)}` });
      toast({ title: 'Sesión finalizada', description: 'Asiento contable creado automáticamente.' });
      setTimeout(() => setFinalizedEntry(null), 5000);
    },
    onError: (e: any) => {
      toast({ title: 'Error', description: e?.toString() ?? 'No se pudo finalizar.', variant: 'destructive' });
    },
  });

  // ─── Handlers ───────────────────────────────────────────────────────────

  const openNew = useCallback((day: Date, hour: number) => {
    const start = setMinutes(setHours(day, hour), 0);
    const end = setMinutes(setHours(day, hour + 1), 0);
    setForm({
      ...EMPTY_FORM,
      start_at: toLocalDatetime(start.toISOString()),
      end_at: toLocalDatetime(end.toISOString()),
    });
    setEditingId(null);
    setPanelOpen(true);
  }, []);

  const openEdit = useCallback((appt: AppointmentResponse) => {
    setForm({
      patient_id: appt.patient_id,
      patient_name: appt.patient_id.slice(0, 8) + '…',
      therapist_id: appt.therapist_id,
      modality: appt.modality,
      start_at: toLocalDatetime(appt.start_at),
      end_at: toLocalDatetime(appt.end_at),
      fee_cents: appt.fee_cents,
      notes: appt.notes ?? '',
      reminder_minutes_before: appt.reminder_minutes_before,
    });
    setEditingId(appt.id);
    setPanelOpen(true);
  }, []);

  const closePanel = useCallback(() => {
    setPanelOpen(false);
    setEditingId(null);
    setForm(EMPTY_FORM);
    setPatientQuery('');
  }, []);

  const handleSubmit = () => {
    const req: CreateAppointmentRequest = {
      patient_id: form.patient_id,
      therapist_id: form.therapist_id || form.patient_id, // fallback
      modality: form.modality,
      start_at: fromLocalDatetime(form.start_at),
      end_at: fromLocalDatetime(form.end_at),
      fee_cents: form.fee_cents,
      notes: form.notes || undefined,
      reminder_minutes_before: form.reminder_minutes_before ?? undefined,
    };

    if (editingId) {
      const updateReq: UpdateAppointmentRequest = {
        modality: form.modality,
        start_at: fromLocalDatetime(form.start_at),
        end_at: fromLocalDatetime(form.end_at),
        fee_cents: form.fee_cents,
        notes: form.notes || undefined,
        reminder_minutes_before: form.reminder_minutes_before ?? undefined,
      };
      updateMutation.mutate({ id: editingId, req: updateReq });
    } else {
      createMutation.mutate(req);
    }
  };

  // ─── Derived ────────────────────────────────────────────────────────────

  const getAppointmentsForDay = useMemo(() => {
    const map = new Map<string, AppointmentResponse[]>();
    for (const day of days) {
      const key = format(day, 'yyyy-MM-dd');
      map.set(
        key,
        appointments.filter((a) => isSameDay(parseISO(a.start_at), day))
      );
    }
    return (day: Date) => map.get(format(day, 'yyyy-MM-dd')) ?? [];
  }, [days, appointments]);

  const isMutating = createMutation.isPending || updateMutation.isPending;

  // ─── Render ─────────────────────────────────────────────────────────────

  return (
    <div className="flex h-[calc(100vh-4rem)] gap-4 p-4 overflow-hidden">
      {/* ── Main calendar area ── */}
      <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between mb-4 flex-shrink-0">
          <div>
            <h1 className="text-2xl font-bold tracking-tight">Agenda</h1>
            <p className="text-sm text-muted-foreground">
              {format(weekStart, "d 'de' MMM", { locale: es })} –{' '}
              {format(weekEnd, "d 'de' MMM yyyy", { locale: es })}
            </p>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="icon" onClick={() => setCurrentWeek(subWeeks(currentWeek, 1))}>
              <ChevronLeft className="h-4 w-4" />
            </Button>
            <Button variant="outline" size="sm" onClick={() => setCurrentWeek(new Date())}>
              <CalendarDays className="mr-1 h-3 w-3" />
              Hoy
            </Button>
            <Button variant="outline" size="icon" onClick={() => setCurrentWeek(addWeeks(currentWeek, 1))}>
              <ChevronRight className="h-4 w-4" />
            </Button>
            <Button size="sm" onClick={() => openNew(new Date(), new Date().getHours())}>
              <Plus className="mr-1 h-3 w-3" />
              Nueva Cita
            </Button>
          </div>
        </div>

        {/* Metrics bar */}
        {metrics && (
          <div className="grid grid-cols-5 gap-3 mb-4 flex-shrink-0">
            <MetricCard icon={<CalendarDays className="h-4 w-4" />} label="Total" value={metrics.total_citas} />
            <MetricCard icon={<Clock className="h-4 w-4" />} label="Programadas" value={metrics.programadas} />
            <MetricCard icon={<CheckCircle2 className="h-4 w-4" />} label="Realizadas" value={metrics.realizadas} />
            <MetricCard
              icon={<TrendingUp className="h-4 w-4" />}
              label="Utilización"
              value={`${Math.round(metrics.utilization_rate * 100)}%`}
            />
            <MetricCard
              icon={<DollarSign className="h-4 w-4" />}
              label="Ingresos"
              value={formatFeeCents(metrics.revenue_cents)}
            />
          </div>
        )}

        {/* Accounting trigger toast */}
        {finalizedEntry && (
          <div className="mb-3 p-3 bg-green-50 border border-green-200 rounded-lg flex items-center gap-2 text-sm text-green-800 animate-in fade-in">
            <CheckCircle2 className="h-4 w-4 text-green-600" />
            <span className="font-medium">Asiento contable creado:</span>
            <span>{finalizedEntry.descripcion}</span>
          </div>
        )}

        {/* Calendar grid */}
        <div className="flex-1 overflow-auto rounded-lg border">
          {/* Day headers */}
          <div className="grid grid-cols-[60px_repeat(7,1fr)] sticky top-0 z-10 bg-background border-b">
            <div className="p-2 text-xs text-muted-foreground" />
            {days.map((day) => {
              const isToday = isSameDay(day, new Date());
              return (
                <div key={day.toISOString()} className={`p-2 text-center text-xs font-medium border-l ${isToday ? 'bg-primary/5 text-primary' : ''}`}>
                  <div>{format(day, 'EEE', { locale: es })}</div>
                  <div className={`text-lg ${isToday ? 'font-bold' : ''}`}>{format(day, 'd')}</div>
                </div>
              );
            })}
          </div>

          {/* Time grid */}
          <div className="grid grid-cols-[60px_repeat(7,1fr)] relative">
            {/* Hour labels + grid lines */}
            {HOURS.map((hour) => (
              <div key={hour} className="contents">
                <div className="h-16 border-b border-r px-1 pt-1 text-[10px] text-muted-foreground text-right pr-2">
                  {String(hour).padStart(2, '0')}:00
                </div>
                {days.map((day, di) => (
                  <div
                    key={`${hour}-${di}`}
                    className="h-16 border-b border-l hover:bg-muted/30 cursor-pointer transition-colors"
                    onClick={() => openNew(day, hour)}
                  />
                ))}
              </div>
            ))}

            {/* Appointment cards overlay */}
            {isLoading ? (
              <div className="col-span-8 p-4">
                <Skeleton className="h-8 w-full mb-2" />
                <Skeleton className="h-8 w-full mb-2" />
                <Skeleton className="h-8 w-3/4" />
              </div>
            ) : (
              days.map((day, di) => {
                const dayAppts = getAppointmentsForDay(day);
                return dayAppts.map((appt) => {
                  const top = slotTop(appt.start_at);
                  const height = slotHeight(appt.start_at, appt.end_at);
                  return (
                    <div
                      key={appt.id}
                      className={`absolute rounded-md border px-1.5 py-1 text-[11px] leading-tight cursor-pointer overflow-hidden transition-shadow hover:shadow-md ${STATUS_COLORS[appt.status] ?? 'bg-gray-100 text-gray-800 border-gray-200'}`}
                      style={{
                        top: `${top + 40}px`, // offset for header row
                        height: `${height}px`,
                        left: `calc(${((di) / 7) * 100}% + 60px)`,
                        width: `calc(100% / 7 - 4px)`,
                      }}
                      onClick={(e) => {
                        e.stopPropagation();
                        openEdit(appt);
                      }}
                    >
                      <div className="font-semibold truncate">
                        {format(parseISO(appt.start_at), 'HH:mm')}–{format(parseISO(appt.end_at), 'HH:mm')}
                      </div>
                      <div className="truncate opacity-80">{appt.patient_id.slice(0, 8)}…</div>
                      <div className="truncate opacity-60">{MODALITY_LABELS[appt.modality]}</div>
                    </div>
                  );
                });
              })
            )}
          </div>
        </div>
      </div>

      {/* ── Side panel (create/edit) ── */}
      {panelOpen && (
        <div className="w-96 flex-shrink-0 border-l bg-background overflow-y-auto">
          <div className="sticky top-0 z-10 bg-background border-b px-4 py-3 flex items-center justify-between">
            <h2 className="font-semibold">{editingId ? 'Editar Cita' : 'Nueva Cita'}</h2>
            <Button variant="ghost" size="icon" onClick={closePanel}>
              <X className="h-4 w-4" />
            </Button>
          </div>

          <div className="p-4 space-y-4">
            {/* Patient search */}
            <div>
              <Label className="text-xs">Paciente *</Label>
              <div className="relative mt-1">
                <Search className="absolute left-2 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  value={patientQuery || form.patient_name}
                  onChange={(e) => {
                    setPatientQuery(e.target.value);
                    setForm((f) => ({ ...f, patient_id: '', patient_name: e.target.value }));
                  }}
                  placeholder="Buscar paciente..."
                  className="pl-8"
                />
                {patientQuery.length >= 2 && patientResults.length > 0 && (
                  <div className="absolute z-50 w-full mt-1 bg-background border rounded-lg shadow-lg max-h-48 overflow-y-auto">
                    {patientResults.map((p: any) => (
                      <button
                        key={p.id}
                        onClick={() => {
                          setForm((f) => ({ ...f, patient_id: p.id, patient_name: `${p.first_name} ${p.last_name}` }));
                          setPatientQuery('');
                        }}
                        className="w-full text-left px-3 py-2 hover:bg-accent text-sm"
                      >
                        {p.first_name} {p.last_name}
                        {p.cedula && <span className="text-muted-foreground ml-2">({p.cedula})</span>}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            </div>

            {/* Modality */}
            <div>
              <Label className="text-xs">Modalidad *</Label>
              <select
                value={form.modality}
                onChange={(e) => setForm((f) => ({ ...f, modality: e.target.value as Modality }))}
                className="mt-1 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              >
                {MODALITIES.map((m) => (
                  <option key={m} value={m}>{MODALITY_LABELS[m]}</option>
                ))}
              </select>
            </div>

            {/* Start / End */}
            <div className="grid grid-cols-2 gap-3">
              <div>
                <Label className="text-xs">Inicio *</Label>
                <Input
                  type="datetime-local"
                  value={form.start_at}
                  onChange={(e) => setForm((f) => ({ ...f, start_at: e.target.value }))}
                  className="mt-1"
                />
              </div>
              <div>
                <Label className="text-xs">Fin *</Label>
                <Input
                  type="datetime-local"
                  value={form.end_at}
                  onChange={(e) => setForm((f) => ({ ...f, end_at: e.target.value }))}
                  className="mt-1"
                />
              </div>
            </div>

            {/* Fee */}
            <div>
              <Label className="text-xs">Honorarios (centavos)</Label>
              <Input
                type="number"
                value={form.fee_cents || ''}
                onChange={(e) => setForm((f) => ({ ...f, fee_cents: parseInt(e.target.value) || 0 }))}
                placeholder="0"
                className="mt-1"
              />
              {form.fee_cents > 0 && (
                <p className="text-xs text-muted-foreground mt-1">{formatFeeCents(form.fee_cents)}</p>
              )}
            </div>

            {/* Reminder */}
            <div>
              <Label className="text-xs">Recordatorio (minutos antes)</Label>
              <Input
                type="number"
                value={form.reminder_minutes_before ?? ''}
                onChange={(e) =>
                  setForm((f) => ({ ...f, reminder_minutes_before: e.target.value ? parseInt(e.target.value) : null }))
                }
                placeholder="30"
                className="mt-1"
              />
            </div>

            {/* Notes */}
            <div>
              <Label className="text-xs">Notas</Label>
              <Textarea
                value={form.notes}
                onChange={(e) => setForm((f) => ({ ...f, notes: e.target.value }))}
                placeholder="Observaciones de la sesión..."
                className="mt-1"
                rows={3}
              />
            </div>

            {/* Actions */}
            <div className="flex gap-2 pt-2">
              <Button className="flex-1" onClick={handleSubmit} disabled={isMutating || !form.patient_id || !form.start_at || !form.end_at}>
                {isMutating && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                {editingId ? 'Guardar' : 'Crear Cita'}
              </Button>
            </div>

            {/* Status-specific actions */}
            {editingId && (
              <div className="flex gap-2 pt-1">
                <Button
                  variant="outline"
                  size="sm"
                  className="flex-1"
                  onClick={() => finalizeMutation.mutate(editingId)}
                  disabled={finalizeMutation.isPending}
                >
                  <CheckCircle2 className="mr-1 h-3 w-3" />
                  Finalizar
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  className="flex-1 text-destructive hover:text-destructive"
                  onClick={() => cancelMutation.mutate(editingId)}
                  disabled={cancelMutation.isPending}
                >
                  <Trash2 className="mr-1 h-3 w-3" />
                  Cancelar
                </Button>
              </div>
            )}

            {/* Status badge */}
            {editingId && (
              <div className="pt-1">
                <Label className="text-xs">Estado</Label>
                <div className="mt-1">
                  {(() => {
                    const appt = appointments.find((a) => a.id === editingId);
                    return appt ? (
                      <Badge variant="outline" className={STATUS_COLORS[appt.status]}>
                        {STATUS_LABELS[appt.status]}
                      </Badge>
                    ) : null;
                  })()}
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

// ─── Metric card sub-component ──────────────────────────────────────────────

function MetricCard({ icon, label, value }: { icon: React.ReactNode; label: string; value: string | number }) {
  return (
    <Card>
      <CardContent className="p-3 flex items-center gap-3">
        <div className="p-2 rounded-md bg-primary/10 text-primary">{icon}</div>
        <div>
          <p className="text-xs text-muted-foreground">{label}</p>
          <p className="text-lg font-semibold">{value}</p>
        </div>
      </CardContent>
    </Card>
  );
}
