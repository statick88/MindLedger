'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';

export function ClinicalNotesPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Historias Clínicas</h1>
        <p className="text-muted-foreground">Gestión de historias clínicas y notas médicas</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Notas Recientes</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {[
              { patient: 'Juan Pérez', doctor: 'Dr. García', type: 'Consulta', date: '2024-01-15' },
              { patient: 'María López', doctor: 'Dra. Fernández', type: 'Control', date: '2024-01-14' },
              { patient: 'Carlos Ruiz', doctor: 'Dr. Rodríguez', type: 'Urgencia', date: '2024-01-13' },
            ].map((note, i) => (
              <div key={i} className="flex items-center justify-between p-4 bg-muted/50 rounded-lg">
                <div>
                  <p className="font-medium">{note.patient}</p>
                  <p className="text-sm text-muted-foreground">{note.type} • {note.doctor} • {note.date}</p>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}