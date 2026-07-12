'use client';

import { ChangeEvent, useState } from 'react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Search } from 'lucide-react';

interface PatientSearchProps {
  value: string;
  onChange: (value: string) => void;
  onSearch: () => void;
}

export function PatientSearch({ value, onChange, onSearch }: PatientSearchProps) {
  const [inputValue, setInputValue] = useState(value);

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    setInputValue(newValue);
    onChange(newValue);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      onSearch();
    }
  };

  return (
    <div className="relative max-w-md">
      <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
      <Input
        type="search"
        placeholder="Buscar por nombre, documento, email..."
        value={inputValue}
        onChange={handleChange}
        onKeyDown={handleKeyDown}
        className="pl-10"
      />
      {inputValue && (
        <Button
          variant="ghost"
          size="icon"
          className="absolute right-2 top-1/2 -translate-y-1/2 h-6 w-6"
          onClick={() => {
            setInputValue('');
            onChange('');
          }}
          aria-label="Limpiar búsqueda"
        >
          <Search className="h-4 w-4" />
        </Button>
      )}
    </div>
  );
}