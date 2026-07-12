'use client';

import { useToast } from '@/hooks/use-toast.tsx';
import { X } from 'lucide-react';
import { cn } from '@/utils/cn';
import { Button } from '@/components/ui/Button';

export function Toaster() {
  const { toasts, dismiss } = useToast();

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2" aria-live="polite">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={cn(
            'flex items-start gap-3 w-full max-w-sm rounded-lg border bg-background p-4 shadow-lg animate-in slide-in-from-right-full',
            toast.variant === 'destructive' && 'border-destructive/50 bg-destructive/10 text-destructive'
          )}
        >
          <div className="flex-1">
            {toast.title && <div className="font-medium">{toast.title}</div>}
            {toast.description && <div className="text-sm text-muted-foreground mt-1">{toast.description}</div>}
          </div>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => dismiss(toast.id)}
            className="h-6 w-6 p-0"
          >
            <X className="h-4 w-4" />
          </Button>
        </div>
      ))}
    </div>
  );
}