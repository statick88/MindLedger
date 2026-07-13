'use client';

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { settingsApi } from '@/api';
import type { Settings } from '@/types';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import { Skeleton } from '@/components/ui/Skeleton';
import { useToast } from '@/hooks/use-toast';
import { translateError } from '@/utils/translate-error';
import { useState, useEffect } from 'react';
import { Save, Loader2, Check } from 'lucide-react';

export function SettingsPage() {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  const { data: settings, isLoading } = useQuery<Settings>({
    queryKey: ['settings'],
    queryFn: () => settingsApi.get(),
  });

  const [form, setForm] = useState<Partial<Settings>>({});
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    if (settings) setForm(settings);
  }, [settings]);

  const updateMutation = useMutation({
    mutationFn: (request: Partial<Settings>) => settingsApi.update(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings'] });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
      toast({
        title: 'Configuración guardada',
        description: 'Los cambios se han aplicado correctamente.',
      });
    },
    onError: (err) => {
      toast({
        title: 'Error al guardar',
        description: translateError(err),
        variant: 'destructive',
      });
    },
  });

  const handleSave = () => {
    updateMutation.mutate(form);
  };

  const updateField = (field: keyof Settings, value: string | number) => {
    setForm((prev) => ({ ...prev, [field]: value }));
  };

  if (isLoading) {
    return (
      <div className="space-y-6 max-w-2xl">
        <div>
          <Skeleton className="h-8 w-40 mb-2" />
          <Skeleton className="h-4 w-56" />
        </div>
        <Card>
          <CardHeader>
            <Skeleton className="h-5 w-48" />
            <Skeleton className="h-3 w-64" />
          </CardHeader>
          <CardContent className="space-y-4">
            {[1, 2].map((i) => (
              <div key={i} className="space-y-2">
                <Skeleton className="h-3 w-28" />
                <Skeleton className="h-10 w-full" />
              </div>
            ))}
            <div className="grid grid-cols-2 gap-4">
              {[1, 2].map((i) => (
                <div key={i} className="space-y-2">
                  <Skeleton className="h-3 w-20" />
                  <Skeleton className="h-10 w-full" />
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <Skeleton className="h-5 w-44" />
            <Skeleton className="h-3 w-48" />
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              {[1, 2, 3, 4].map((i) => (
                <div key={i} className="space-y-2">
                  <Skeleton className="h-3 w-24" />
                  <Skeleton className="h-10 w-full" />
                </div>
              ))}
            </div>
            <div className="space-y-2">
              <Skeleton className="h-3 w-16" />
              <Skeleton className="h-10 w-full" />
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6 max-w-2xl">
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-primary">Configuración</h1>
        <p className="text-muted-foreground">Ajustes generales de la clínica</p>
      </div>

      {/* Clinic Info */}
      <Card>
        <CardHeader>
          <CardTitle>Información de la Clínica</CardTitle>
          <CardDescription>Datos que aparecen en reportes y documentos</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="clinic_name">Nombre de la Clínica</Label>
            <Input
              id="clinic_name"
              value={form.clinic_name ?? ''}
              onChange={(e) => updateField('clinic_name', e.target.value)}
              placeholder="Mi Clínica"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="clinic_address">Dirección</Label>
            <Input
              id="clinic_address"
              value={form.clinic_address ?? ''}
              onChange={(e) => updateField('clinic_address', e.target.value)}
              placeholder="Av. Principal 123, Quito"
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="clinic_phone">Teléfono</Label>
              <Input
                id="clinic_phone"
                value={form.clinic_phone ?? ''}
                onChange={(e) => updateField('clinic_phone', e.target.value)}
                placeholder="+593 99 123 4567"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="clinic_email">Email</Label>
              <Input
                id="clinic_email"
                type="email"
                value={form.clinic_email ?? ''}
                onChange={(e) => updateField('clinic_email', e.target.value)}
                placeholder="contacto@miclinica.com"
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Operational Settings */}
      <Card>
        <CardHeader>
          <CardTitle>Configuración Operativa</CardTitle>
          <CardDescription>Ajustes de operación diaria</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="timezone">Zona Horaria</Label>
              <Input
                id="timezone"
                value={form.timezone ?? ''}
                onChange={(e) => updateField('timezone', e.target.value)}
                placeholder="America/Guayaquil"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="currency">Moneda</Label>
              <Input
                id="currency"
                value={form.currency ?? ''}
                onChange={(e) => updateField('currency', e.target.value)}
                placeholder="USD"
              />
            </div>
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="appointment_duration_default">Duración Cita (min)</Label>
              <Input
                id="appointment_duration_default"
                type="number"
                value={form.appointment_duration_default ?? 30}
                onChange={(e) => updateField('appointment_duration_default', parseInt(e.target.value) || 30)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="age_of_majority">Edad de Mayoría</Label>
              <Input
                id="age_of_majority"
                type="number"
                value={form.age_of_majority ?? 18}
                onChange={(e) => updateField('age_of_majority', parseInt(e.target.value) || 18)}
              />
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="language">Idioma</Label>
            <Input
              id="language"
              value={form.language ?? ''}
              onChange={(e) => updateField('language', e.target.value)}
              placeholder="es"
            />
          </div>
        </CardContent>
      </Card>

      {/* Save */}
      <div className="flex justify-end gap-3">
        <Button
          variant="outline"
          onClick={() => settings && setForm(settings)}
        >
          Restablecer
        </Button>
        <Button
          onClick={handleSave}
          disabled={updateMutation.isPending}
          className="gap-2"
        >
          {updateMutation.isPending ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : saved ? (
            <Check className="h-4 w-4" />
          ) : (
            <Save className="h-4 w-4" />
          )}
          {saved ? 'Guardado' : 'Guardar Cambios'}
        </Button>
      </div>
    </div>
  );
}
