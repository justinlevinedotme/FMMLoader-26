import { useState, useCallback } from 'react';
import { Upload } from 'lucide-react';
import { cn } from '@/lib/utils';

interface DropZoneProps {
  onDrop: (files: FileList) => void;
  className?: string;
}

export function DropZone({ onDrop, className }: DropZoneProps) {
  const [isDragging, setIsDragging] = useState(false);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragging(false);

      const files = e.dataTransfer.files;
      if (files && files.length > 0) {
        onDrop(files);
      }
    },
    [onDrop]
  );

  return (
    <div
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      className={cn(
        'border-2 border-dashed rounded-lg p-8 text-center transition-colors',
        isDragging
          ? 'border-primary bg-primary/5'
          : 'border-muted-foreground/25 hover:border-muted-foreground/50',
        className
      )}
    >
      <Upload
        className={cn(
          'mx-auto h-12 w-12 mb-4 transition-colors',
          isDragging ? 'text-primary' : 'text-muted-foreground'
        )}
      />
      <p className="text-lg font-medium mb-1">
        {isDragging ? 'Drop your mod files here' : 'Drag & drop mod files'}
      </p>
      <p className="text-sm text-muted-foreground">
        Supports .zip files, folders, and individual mod files
      </p>
    </div>
  );
}
