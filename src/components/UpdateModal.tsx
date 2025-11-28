import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useI18n } from '@/lib/i18n';
import type { UpdateStatus } from '@/hooks/useUpdater';

type UpdateModalProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  status: UpdateStatus;
  appVersion: string;
  onDownload: () => void;
};

export function UpdateModal({
  open,
  onOpenChange,
  status,
  appVersion,
  onDownload,
}: UpdateModalProps) {
  const { t } = useI18n();
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('settings.updates.modalTitle')}</DialogTitle>
          <DialogDescription>
            {status.latestVersion
              ? t('settings.updates.modalBody', {
                  latest: status.latestVersion,
                  current: (status.currentVersion ?? appVersion) || 'unknown',
                })
              : t('settings.updates.modalTitle')}
          </DialogDescription>
        </DialogHeader>
        {status.error && <p className="text-sm text-red-500">{status.error}</p>}
        <div className="flex justify-end gap-2 pt-4">
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('settings.updates.later')}
          </Button>
          <Button onClick={onDownload} disabled={status.downloading || status.installing}>
            {status.downloading
              ? t('settings.updates.downloading')
              : status.installing
                ? t('settings.updates.installing')
                : t('settings.updates.downloadInstall')}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
