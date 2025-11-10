import { useState } from 'react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Download, X } from 'lucide-react';
import { open as openUrl } from '@tauri-apps/plugin-shell';
import type { UpdateInfo } from '@/hooks/useTauri';

interface UpdateBannerProps {
  updateInfo: UpdateInfo;
  onDismiss: () => void;
}

export function UpdateBanner({ updateInfo, onDismiss }: UpdateBannerProps) {
  const [dismissed, setDismissed] = useState(false);

  if (dismissed || !updateInfo.has_update) {
    return null;
  }

  const handleDownload = async () => {
    try {
      await openUrl(updateInfo.download_url);
    } catch (error) {
      console.error('Failed to open download URL:', error);
    }
  };

  const handleDismiss = () => {
    setDismissed(true);
    onDismiss();
  };

  return (
    <div className="mx-4 mt-4 pt-10 ">
      <Alert className="relative">
        <Download className="h-4 w-4" />
        <AlertTitle>Update Available</AlertTitle>
        <AlertDescription className="flex items-center justify-between">
          <span>
            Version {updateInfo.latest_version} is now available! You&apos;re currently running{' '}
            {updateInfo.current_version}.
          </span>
          <div className="flex items-center gap-2 ml-4">
            <Button size="sm" onClick={handleDownload}>
              Download
            </Button>
            <Button variant="ghost" size="icon" onClick={handleDismiss} className="h-6 w-6">
              <X className="h-4 w-4" />
            </Button>
          </div>
        </AlertDescription>
      </Alert>
    </div>
  );
}
