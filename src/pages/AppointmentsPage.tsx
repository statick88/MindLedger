"use client";

import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { Plus, ChevronLeft, ChevronRight, CalendarDays } from "lucide-react";
import { format, startOfWeek, endOfWeek, addWeeks, subWeeks, isSameDay } from "date-fns";
import { es } from "date-fns/locale";

export function AppointmentsPage() {
  const [currentWeek, setCurrentWeek] = useState<Date>(new Date());

  const weekStart = startOfWeek(currentWeek, { weekStartsOn: 1 });
  const weekEnd = endOfWeek(currentWeek, { weekStartsOn: 1 });

  const days = Array.from({ length: 7 }, (_, i) => {
    const date = new Date(weekStart);
    date.setDate(date.getDate() + i);
    return date;
  });

  const appointments = [
    { id: 1, patient: "Juan Perez", doctor: "Dr. Garcia", time: "09:00", duration: 30, type: "Consulta", day: 0 },
    { id: 2, patient: "Maria Lopez", doctor: "Dra. Fernandez", time: "10:30", duration: 30, type: "Control", day: 0 },
    { id: 3, patient: "Carlos Ruiz", doctor: "Dr. Rodriguez", time: "11:00", duration: 45, type: "Urgencia", day: 1 },
    { id: 4, patient: "Ana Gonzalez", doctor: "Dra. Martinez", time: "14:00", duration: 30, type: "Consulta", day: 2 },
  ];

  const getAppointmentsForDay = (dayIndex: number) => {
    return appointments.filter((a) => a.day === dayIndex).sort((a, b) => a.time.localeCompare(b.time));
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Agenda de Turnos</h1>
          <p className="text-muted-foreground">
            Semana del {format(weekStart, "d MMM", { locale: es })} al {format(weekEnd, "d MMM yyyy", { locale: es })}
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="icon" onClick={() => setCurrentWeek(subWeeks(currentWeek, 1))}>
            <ChevronLeft className="h-4 w-4" />
          </Button>
          <Button variant="outline" size="icon" onClick={() => setCurrentWeek(new Date())}>
            <CalendarDays className="h-4 w-4" />
          </Button>
          <Button variant="outline" size="icon" onClick={() => setCurrentWeek(addWeeks(currentWeek, 1))}>
            <ChevronRight className="h-4 w-4" />
          </Button>
          <Button>
            <Plus className="mr-2 h-4 w-4" />
            Nuevo Turno
          </Button>
        </div>
      </div>

      <div className="grid gap-4" style={{ gridTemplateColumns: "repeat(7, 1fr)" }}>
        {days.map((day, index) => {
          const isToday = isSameDay(day, new Date());
          return (
            <Card key={index} className={isToday ? "ring-2 ring-primary" : ""}>
              <CardHeader className="pb-2">
                <CardTitle className="text-center text-sm">
                  <div className={isToday ? "text-primary font-bold" : ""}>
                    {format(day, "EEE", { locale: es })}
                  </div>
                  <div className={isToday ? "text-primary font-bold" : ""}>
                    {format(day, "d", { locale: es })}
                  </div>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2 min-h-[400px]">
                  {getAppointmentsForDay(index).map((appt) => (
                    <div
                      key={appt.id}
                      className="p-2 bg-muted/50 rounded-lg border hover:bg-muted/80 transition-colors cursor-pointer"
                    >
                      <div className="font-medium text-sm">{appt.time}</div>
                      <div className="text-xs text-muted-foreground">{appt.patient}</div>
                      <div className="text-xs text-muted-foreground">{appt.doctor}</div>
                      <span className="inline-block mt-1 px-1.5 py-0.5 text-xs bg-primary/10 text-primary rounded">
                        {appt.type}
                      </span>
                    </div>
                  ))}
                  <Button variant="ghost" size="sm" className="w-full mt-2">
                    <Plus className="mr-1 h-3 w-3" />
                    Agregar
                  </Button>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}