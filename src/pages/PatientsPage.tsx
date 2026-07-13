'use client';

import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { patientApi, ageApi } from '@/api';
import type { Patient } from '@/types';
import type { AgeBreakdown } from '@/api';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Badge } from '@/components/ui/Badge';
import { Skeleton } from '@/components/ui/Skeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { translateError } from '@/utils/translate-error';
import { useToast } from '@/hooks/use-toast';
import {
  Users,
  Search,
  Plus,
  ChevronLeft,
  ChevronRight,
  Calendar,
  FileText,
} from 'lucide-react';

// ─── Age Display ────────────────────────────────────────────────────────────

function PatientAge({ dateOfBirth }: { dateOfBirth: string }) {
  const { data: ageBreakdown, isLoading } = useQuery<AgeBreakdown>({
    queryKey: ['age-breakdown', dateOfBirth],
    queryFn: () => ageApi.breakdown(dateOfBirth),
    staleTime: 1000 * 60 * 60, // 1 hour — age doesn't change often
  });

  if (isLoading) return <span className="text-muted-foreground">...</span>;
  if (!ageBreakdown) return <span className="text-muted-foreground">—</span>;

  return (
    <span className={ageBreakdown.is_minor ? 'text-coral font-medium' : ''}>
      {ageBreakdown.years} años
      {ageBreakdown.months > 0 && `, ${ageBreakdown.months}m`}
      {ageBreakdown.is_minor && ' (menor)'}
    </span>
  );
}

// ─── Patient List ───────────────────────────────────────────────────────────

const PAGE_SIZE = 10;

export function PatientsPage() {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  const [searchQuery, setSearchQuery] = useState('');
  const [page, setPage] = useState(1);
  const [activeFilter, setActiveFilter] = useState<boolean | undefined>(true);

  const { data, isLoading, error } = useQuery({
    queryKey: ['patients', page, activeFilter],
    queryFn: () =>
      searchQuery
        ? patientApi.search(searchQuery, page, PAGE_SIZE)
        : patientApi.list({ page, page_size: PAGE_SIZE, active_only: activeFilter }),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => patientApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['patients'] });
      toast({
        title: 'Paciente eliminado',
        description: 'El paciente ha sido desactivado correctamente.',
      });
    },
    onError: (err) => {
      toast({
        title: 'Error al eliminar',
        description: translateError(err),
        variant: 'destructive',
      });
    },
  });

  const patients = data?.items ?? [];
  const totalPages = data?.total_pages ?? 1;
  const total = data?.total ?? 0;

  const handleSearch = () => {
    setPage(1);
    // React Query will refetch with the new search query
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-primary">Pacientes</h1>
          <p className="text-muted-foreground">
            {total} paciente{total !== 1 ? 's' : ''} registrado{total !== 1 ? 's' : ''}
          </p>
        </div>
        <Button className="gap-2">
          <Plus className="h-4 w-4" />
          Nuevo Paciente
        </Button>
      </div>

      {/* Search + Filters */}
      <Card>
        <CardContent className="p-4">
          <div className="flex flex-col sm:flex-row gap-3">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Buscar por nombre, cédula, email..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
                className="pl-9"
              />
            </div>
            <div className="flex gap-2">
              <Button
                variant={activeFilter === true ? 'default' : 'outline'}
                size="sm"
                onClick={() => { setActiveFilter(true); setPage(1); }}
              >
                Activos
              </Button>
              <Button
                variant={activeFilter === undefined ? 'default' : 'outline'}
                size="sm"
                onClick={() => { setActiveFilter(undefined); setPage(1); }}
              >
                Todos
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Loading State */}
      {isLoading && (
        <div className="space-y-3">
          {[1, 2, 3, 4, 5].map((i) => (
            <Card key={i}>
              <CardContent className="p-4">
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1 space-y-2">
                    <div className="flex items-center gap-2">
                      <Skeleton className="h-5 w-40" />
                      <Skeleton className="h-5 w-14 rounded-full" />
                    </div>
                    <div className="flex items-center gap-4">
                      <Skeleton className="h-3 w-28" />
                      <Skeleton className="h-3 w-20" />
                      <Skeleton className="h-3 w-16" />
                    </div>
                    <div className="flex items-center gap-4">
                      <Skeleton className="h-3 w-36" />
                      <Skeleton className="h-3 w-24" />
                    </div>
                  </div>
                  <div className="flex gap-1">
                    <Skeleton className="h-8 w-8 rounded" />
                    <Skeleton className="h-8 w-8 rounded" />
                    <Skeleton className="h-8 w-8 rounded" />
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Error State */}
      {error && (
        <Card>
          <CardContent className="p-6 text-center">
            <p className="text-destructive">{translateError(error)}</p>
          </CardContent>
        </Card>
      )}

      {/* Patient List */}
      {!isLoading && !error && (
        <div className="space-y-3">
          {patients.length === 0 ? (
            <EmptyState
              icon={<Users className="h-8 w-8" />}
              title={searchQuery ? 'Sin resultados' : 'Sin pacientes'}
              description={
                searchQuery
                  ? 'No se encontraron pacientes con esos términos. Intenta con otros.'
                  : 'Registra tu primer paciente para comenzar a gestionar tu consultorio.'
              }
              action={
                !searchQuery
                  ? { label: 'Nuevo Paciente', onClick: () => {} }
                  : undefined
              }
            />
          ) : (
            patients.map((patient) => (
              <PatientCard
                key={patient.id}
                patient={patient}
                onDelete={() => {
                  if (window.confirm(`¿Eliminar a ${patient.first_name} ${patient.last_name}?`)) {
                    deleteMutation.mutate(patient.id);
                  }
                }}
              />
            ))
          )}
        </div>
      )}

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="flex items-center justify-between">
          <p className="text-sm text-muted-foreground">
            Página {page} de {totalPages}
          </p>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setPage((p) => Math.max(1, p - 1))}
              disabled={page <= 1}
            >
              <ChevronLeft className="h-4 w-4" />
              Anterior
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
              disabled={page >= totalPages}
            >
              Siguiente
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}

// ─── Patient Card ───────────────────────────────────────────────────────────

function PatientCard({
  patient,
  onDelete,
}: {
  patient: Patient;
  onDelete: () => void;
}) {
  const genderLabel: Record<string, string> = {
    Male: 'Masculino',
    Female: 'Femenino',
    Other: 'Otro',
    PreferNotToSay: 'Prefiere no decir',
  };

  return (
    <Card className="hover:shadow-md transition-shadow">
      <CardContent className="p-4">
        <div className="flex items-start justify-between gap-4">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <h3 className="font-semibold text-lg truncate">
                {patient.first_name} {patient.middle_name} {patient.last_name}
              </h3>
              <Badge variant={patient.is_active ? 'default' : 'secondary'}>
                {patient.is_active ? 'Activo' : 'Inactivo'}
              </Badge>
              {patient.allergies?.length > 0 && (
                <Badge variant="destructive">Alergias</Badge>
              )}
            </div>

            <div className="mt-1 flex items-center gap-4 text-sm text-muted-foreground flex-wrap">
              <span>{patient.document_type}: {patient.document_number}</span>
              <span className="flex items-center gap-1">
                <Calendar className="h-3 w-3" />
                <PatientAge dateOfBirth={patient.date_of_birth} />
              </span>
              <span>{genderLabel[patient.gender] ?? patient.gender}</span>
            </div>

            <div className="mt-1 flex items-center gap-4 text-sm text-muted-foreground flex-wrap">
              {patient.email && <span>{patient.email}</span>}
              {patient.phone && <span>{patient.phone}</span>}
            </div>

            {patient.chronic_conditions?.length > 0 && (
              <div className="mt-2 flex gap-1 flex-wrap">
                {patient.chronic_conditions.map((condition, i) => (
                  <Badge key={i} variant="outline" className="text-xs">
                    {condition}
                  </Badge>
                ))}
              </div>
            )}
          </div>

          <div className="flex gap-1 flex-shrink-0">
            <Button variant="ghost" size="icon" title="Historia Clínica">
              <FileText className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" title="Agendar Cita">
              <Calendar className="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              title="Eliminar"
              onClick={onDelete}
              className="text-destructive hover:text-destructive"
            >
              <Users className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
