import { useState, useEffect } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { tauriCommands } from '@/hooks/useTauri';
import { useI18n } from '@/lib/i18n';
import { logger } from '@/lib/logger';
import type { ModMetadata } from '@/types';

interface ModMetadataDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  sourcePath: string;
  onSubmit: (metadata: ModMetadata) => void;
}

const MOD_TYPES = [
  { value: 'ui', label: 'UI' },
  { value: 'bundle', label: 'Bundle' },
  { value: 'tactics', label: 'Tactics' },
  { value: 'graphics', label: 'Graphics' },
  { value: 'skins', label: 'Skins' },
  { value: 'editor-data', label: 'Editor Data' },
  { value: 'misc', label: 'Miscellaneous' },
];

export function ModMetadataDialog({
  open,
  onOpenChange,
  sourcePath,
  onSubmit,
}: ModMetadataDialogProps) {
  const { t } = useI18n();
  const [name, setName] = useState('');
  const [version, setVersion] = useState('1.0.0');
  const [modType, setModType] = useState('misc');
  const [author, setAuthor] = useState('');
  const [description, setDescription] = useState('');
  const [detecting, setDetecting] = useState(false);

  // Auto-detect mod type when dialog opens
  useEffect(() => {
    if (open && sourcePath) {
      const detectType = async () => {
        setDetecting(true);
        try {
          const detectedType = await tauriCommands.detectModType(sourcePath);
          setModType(detectedType);
        } catch (err) {
          logger.error('Failed to detect mod type', { error: err, sourcePath });
        } finally {
          setDetecting(false);
        }
      };

      void detectType();

      // Extract a default name from the path
      const pathParts = sourcePath.split(/[/\\]/);
      const lastPart = pathParts[pathParts.length - 1];
      const nameWithoutExt = lastPart.replace(/\.(zip|bundle|fmf)$/i, '');
      setName(nameWithoutExt);
    }
  }, [open, sourcePath]);

  const handleSubmit = () => {
    if (!name || !version || !modType) {
      return;
    }

    onSubmit({
      name,
      version,
      mod_type: modType,
      author: author || undefined,
      description: description || undefined,
    });

    // Reset form
    setName('');
    setVersion('1.0.0');
    setModType('misc');
    setAuthor('');
    setDescription('');
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>{t('modMeta.title')}</DialogTitle>
          <DialogDescription>{t('modMeta.description')}</DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="name">
              {t('modMeta.fields.name.label')} <span className="text-destructive">*</span>
            </Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={t('modMeta.fields.name.placeholder')}
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="version">
              {t('modMeta.fields.version.label')} <span className="text-destructive">*</span>
            </Label>
            <Input
              id="version"
              value={version}
              onChange={(e) => setVersion(e.target.value)}
              placeholder={t('modMeta.fields.version.placeholder')}
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="type">
              {t('modMeta.fields.type.label')} <span className="text-destructive">*</span>
            </Label>
            <Select value={modType} onValueChange={setModType} disabled={detecting}>
              <SelectTrigger id="type">
                <SelectValue placeholder={t('modMeta.fields.type.placeholder')} />
              </SelectTrigger>
              <SelectContent>
                {MOD_TYPES.map((type) => (
                  <SelectItem key={type.value} value={type.value}>
                    {type.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {detecting && (
              <p className="text-xs text-muted-foreground">{t('modMeta.fields.type.detecting')}</p>
            )}
          </div>

          <div className="grid gap-2">
            <Label htmlFor="author">{t('modMeta.fields.author.label')}</Label>
            <Input
              id="author"
              value={author}
              onChange={(e) => setAuthor(e.target.value)}
              placeholder={t('modMeta.fields.author.placeholder')}
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="description">{t('modMeta.fields.description.label')}</Label>
            <Input
              id="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder={t('modMeta.fields.description.placeholder')}
            />
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('modMeta.actions.cancel')}
          </Button>
          <Button onClick={handleSubmit} disabled={!name || !version || !modType}>
            {t('modMeta.actions.submit')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
